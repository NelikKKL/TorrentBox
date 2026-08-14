//! Движок TorrentBox: тонкая, но самодостаточная обвязка над `librqbit::Session`.
//!
//! librqbit асинхронный (tokio), а egui рисует синхронно и в своём цикле,
//! поэтому мы поднимаем отдельный поток с собственным tokio-рантаймом,
//! общаемся с ним через простой канал команд и раз в ~500мс публикуем
//! «снимок» состояния всех торрентов в `Arc<Mutex<...>>`, который UI-поток
//! просто читает на каждом кадре.
//!
//! ВАЖНО ДЛЯ РАЗРАБОТЧИКА, СОБИРАЮЩЕГО ПРОЕКТ:
//! librqbit — активно развивающаяся библиотека, и часть здесь используемых
//! методов (`ManagedTorrent::stats/name/info_hash/is_paused`,
//! `Session::add_torrent/with_torrents/pause/unpause/delete`) сверена по
//! документации docs.rs/librqbit на момент написания (версия 8.x), но если
//! после `cargo build` компилятор укажет на несовпадение сигнатур —
//! почти наверняка вышла новая версия крейта. Смотрите
//! `cargo doc -p librqbit --open` и правьте только этот файл: весь
//! остальной код общается с движком через `TorrentRow` и ничего не знает
//! про librqbit напрямую.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use librqbit::api::TorrentIdOrHash;
use librqbit::{AddTorrent, AddTorrentOptions, Session, SessionOptions, SessionPersistenceConfig};

use crate::models::{FileRow, TorrentRow, TorrentStatus};

pub enum Command {
    AddMagnet { magnet_or_url: String, save_path: Option<PathBuf> },
    AddTorrentFile { path: PathBuf, save_path: Option<PathBuf> },
    Pause(usize),
    Resume(usize),
    Remove { id: usize, delete_files: bool },
    PauseAll,
    ResumeAll,
}

#[derive(Default, Clone)]
pub struct EngineState {
    pub ready: bool,
    pub torrents: Vec<TorrentRow>,
    pub last_error: Option<String>,
    pub listen_port: Option<u16>,
}

pub struct Engine {
    cmd_tx: std_mpsc::Sender<Command>,
    pub state: Arc<Mutex<EngineState>>,
}

impl Engine {
    /// Запускает фоновый поток с tokio-рантаймом и сессией librqbit.
    pub fn spawn(default_download_dir: PathBuf) -> Self {
        let (cmd_tx, cmd_rx) = std_mpsc::channel::<Command>();
        let state = Arc::new(Mutex::new(EngineState::default()));
        let state_bg = state.clone();

        std::thread::Builder::new()
            .name("torrentbox-engine".into())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("не удалось создать tokio runtime");
                rt.block_on(run_engine(default_download_dir, cmd_rx, state_bg));
            })
            .expect("не удалось запустить фоновый поток движка");

        Engine { cmd_tx, state }
    }

    pub fn snapshot(&self) -> EngineState {
        self.state.lock().unwrap().clone()
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.cmd_tx.send(cmd);
    }
}

/// Системная папка для служебных файлов приложения — не путать с папкой
/// загрузок. На Windows это `%APPDATA%\TorrentBox`, на Linux —
/// `~/.local/share/torrentbox`, на macOS — `~/Library/Application
/// Support/TorrentBox`. Здесь хранится только состояние сессии (список
/// торрентов и прогресс докачки), сами скачанные файлы сюда не попадают.
fn app_state_dir() -> PathBuf {
    if let Some(proj) = directories::ProjectDirs::from("", "", "TorrentBox") {
        proj.data_dir().to_path_buf()
    } else {
        // Крайний случай: не удалось определить домашнюю папку пользователя.
        // Используем локальную относительную папку, лишь бы не падать совсем.
        PathBuf::from(".torrentbox-state")
    }
}

async fn run_engine(
    default_download_dir: PathBuf,
    cmd_rx: std_mpsc::Receiver<Command>,
    state: Arc<Mutex<EngineState>>,
) {
    let _ = std::fs::create_dir_all(&default_download_dir);

    // Папка для файла состояния сессии (список торрентов переживает перезапуск),
    // как автодокачка/автозагрузка в LibreTorrent.
    //
    // ВАЖНО: раньше эта папка создавалась внутри папки загрузок
    // (default_download_dir.join(".torrentbox-state")). Это было плохо по
    // двум причинам:
    //  1) в папке загрузок появлялась лишняя служебная папка, не имеющая
    //     отношения к самим скачанным файлам;
    //  2) если папка загрузок — это Windows-папка "Загрузки", перенаправленная
    //     в OneDrive (частая ситуация по умолчанию), то сразу после создания
    //     новой подпапки запись в неё может упасть с "os error 3: Системе не
    //     удается найти указанный путь" — OneDrive не всегда успевает
    //     материализовать папку до того, как мы пытаемся в неё писать.
    // Поэтому храним состояние сессии в системной папке данных приложения
    // (например, %APPDATA%\TorrentBox на Windows, ~/.local/share/torrentbox
    // на Linux) — она не связана с папкой загрузок и не синхронизируется
    // облачными клиентами.
    let persistence_dir = app_state_dir().join("session");
    let _ = std::fs::create_dir_all(&persistence_dir);

    let opts = SessionOptions {
        persistence: Some(SessionPersistenceConfig::Json {
            folder: Some(persistence_dir),
        }),
        ..Default::default()
    };

    let session = match Session::new_with_opts(default_download_dir.clone(), opts).await {
        Ok(s) => s,
        Err(e) => {
            state.lock().unwrap().last_error =
                Some(format!("Не удалось запустить torrent-сессию: {e:#}"));
            return;
        }
    };

    {
        let mut st = state.lock().unwrap();
        st.ready = true;
        st.listen_port = session.tcp_listen_port();
    }

    // Папка загрузки, которую задал пользователь для конкретного добавляемого
    // торрента (иначе используется default_download_dir всей сессии).
    let mut per_torrent_save_path: HashMap<usize, PathBuf> = HashMap::new();

    loop {
        // Обрабатываем все накопившиеся команды от UI, не блокируясь.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::AddMagnet { magnet_or_url, save_path } => {
                    let opts = save_path.clone().map(|p| AddTorrentOptions {
                        output_folder: Some(p.display().to_string()),
                        ..Default::default()
                    });
                    match session
                        .add_torrent(AddTorrent::from_url(magnet_or_url.trim()), opts)
                        .await
                    {
                        Ok(resp) => {
                            if let Some(handle) = resp.into_handle() {
                                if let Some(p) = save_path {
                                    per_torrent_save_path.insert(handle.id(), p);
                                }
                            }
                        }
                        Err(e) => {
                            state.lock().unwrap().last_error =
                                Some(format!("Не удалось добавить магнет-ссылку: {e:#}"));
                        }
                    }
                }
                Command::AddTorrentFile { path, save_path } => {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let opts = save_path.clone().map(|p| AddTorrentOptions {
                                output_folder: Some(p.display().to_string()),
                                ..Default::default()
                            });
                            match session.add_torrent(AddTorrent::from_bytes(bytes), opts).await {
                                Ok(resp) => {
                                    if let Some(handle) = resp.into_handle() {
                                        if let Some(p) = save_path {
                                            per_torrent_save_path.insert(handle.id(), p);
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.lock().unwrap().last_error =
                                        Some(format!("Не удалось добавить .torrent файл: {e:#}"));
                                }
                            }
                        }
                        Err(e) => {
                            state.lock().unwrap().last_error =
                                Some(format!("Не удалось прочитать файл {}: {e}", path.display()));
                        }
                    }
                }
                Command::Pause(id) => {
                    if let Some(t) = session.get(TorrentIdOrHash::Id(id)) {
                        let _ = session.pause(&t).await;
                    }
                }
                Command::Resume(id) => {
                    if let Some(t) = session.get(TorrentIdOrHash::Id(id)) {
                        let _ = session.unpause(&t).await;
                    }
                }
                Command::Remove { id, delete_files } => {
                    let _ = session.delete(TorrentIdOrHash::Id(id), delete_files).await;
                    per_torrent_save_path.remove(&id);
                }
                Command::PauseAll => {
                    let handles = session.with_torrents(|iter| {
                        iter.map(|(_, t)| t.clone()).collect::<Vec<_>>()
                    });
                    for t in handles {
                        let _ = session.pause(&t).await;
                    }
                }
                Command::ResumeAll => {
                    let handles = session.with_torrents(|iter| {
                        iter.map(|(_, t)| t.clone()).collect::<Vec<_>>()
                    });
                    for t in handles {
                        let _ = session.unpause(&t).await;
                    }
                }
            }
        }

        // Собираем свежий снимок всех торрентов для UI.
        let rows: Vec<TorrentRow> = session.with_torrents(|iter| {
            iter.map(|(id, t)| {
                let stats = t.stats();
                let name = t.name().unwrap_or_else(|| format!("Торрент #{id}"));
                let info_hash = t.info_hash().as_string();
                let is_paused = t.is_paused();
                let state_dbg = format!("{:?}", stats.state);

                let total = stats.total_bytes;
                let done = stats.progress_bytes.min(total.max(stats.progress_bytes));
                let progress = if total > 0 {
                    (done as f32 / total as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let (download_speed, upload_speed, eta) = match &stats.live {
                    Some(live) => (
                        live.download_speed.to_string(),
                        live.upload_speed.to_string(),
                        live.time_remaining
                            .as_ref()
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "∞".to_string()),
                    ),
                    None => ("0 B/s".to_string(), "0 B/s".to_string(), "—".to_string()),
                };

                let status = derive_status(
                    &state_dbg,
                    is_paused,
                    stats.finished,
                    stats.error.is_some(),
                );

                let save_path = per_torrent_save_path
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| default_download_dir.clone());

                let files: Vec<FileRow> = Vec::new(); // см. README: список файлов — TODO расширения

                TorrentRow {
                    id,
                    name,
                    info_hash,
                    status,
                    progress,
                    downloaded_bytes: stats.progress_bytes,
                    total_bytes: stats.total_bytes,
                    uploaded_bytes: stats.uploaded_bytes,
                    download_speed,
                    upload_speed,
                    eta,
                    save_path,
                    error: stats.error.clone(),
                    files,
                }
            })
            .collect()
        });

        {
            let mut st = state.lock().unwrap();
            st.torrents = rows;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Переводит отладочное представление `TorrentStatsState` (плюс несколько
/// независимых флагов) в наш собственный `TorrentStatus`. Сделано через
/// сопоставление подстрок в `{:?}`, а не через точные варианты enum,
/// специально: так код остаётся рабочим, даже если librqbit слегка
/// переименует варианты состояния между версиями.
fn derive_status(state_debug: &str, is_paused: bool, finished: bool, has_error: bool) -> TorrentStatus {
    if has_error {
        return TorrentStatus::Error;
    }
    if is_paused {
        return TorrentStatus::Paused;
    }
    if finished {
        return TorrentStatus::Finished;
    }
    let s = state_debug.to_ascii_lowercase();
    if s.contains("init") || s.contains("check") {
        TorrentStatus::Checking
    } else if s.contains("live") {
        TorrentStatus::Downloading
    } else if s.contains("pause") {
        TorrentStatus::Paused
    } else if s.contains("error") {
        TorrentStatus::Error
    } else {
        TorrentStatus::Connecting
    }
}
