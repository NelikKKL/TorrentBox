use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use crate::engine::{Command, Engine};
use crate::models::TorrentRow;
use crate::theme::{self, ThemeMode};
use crate::ui::{
    self, AddDialogResult, AddDialogState, DetailsAction, Filter, ListAction, SettingsResult,
    SettingsState, ToolbarAction,
};

const STORAGE_KEY: &str = "torrentbox-prefs";

#[derive(serde::Serialize, serde::Deserialize)]
struct Prefs {
    dark_mode: bool,
    download_dir: PathBuf,
}

pub struct TorrentBoxApp {
    engine: Engine,
    theme_mode: ThemeMode,
    filter: Filter,
    search: String,
    selected: Option<usize>,

    show_add_dialog: bool,
    add_dialog_state: AddDialogState,

    show_settings: bool,
    settings_state: SettingsState,

    download_dir: PathBuf,
    theme_applied: bool,
    ipc_rx: mpsc::Receiver<String>,
}

impl TorrentBoxApp {
    pub fn new(cc: &eframe::CreationContext<'_>, ipc_rx: mpsc::Receiver<String>) -> Self {
        let prefs: Prefs = cc
            .storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY))
            .unwrap_or_else(|| Prefs {
                dark_mode: false,
                download_dir: default_download_dir(),
            });

        let engine = Engine::spawn(prefs.download_dir.clone());

        Self {
            engine,
            theme_mode: if prefs.dark_mode { ThemeMode::Dark } else { ThemeMode::Light },
            filter: Filter::All,
            search: String::new(),
            selected: None,
            show_add_dialog: false,
            add_dialog_state: AddDialogState::new(prefs.download_dir.clone()),
            show_settings: false,
            settings_state: SettingsState {
                download_dir: prefs.download_dir.clone(),
                dark_mode: prefs.dark_mode,
                listen_port: None,
                register_message: None,
            },
            download_dir: prefs.download_dir,
            theme_applied: false,
            ipc_rx,
        }
    }

    /// Разбирает строку, полученную из argv или по локальному IPC (клик по
    /// magnet-ссылке / двойной клик по .torrent файлу), и добавляет торрент.
    fn open_uri_or_path(&mut self, raw: &str) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return;
        }
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("magnet:") || lower.starts_with("http://") || lower.starts_with("https://") {
            self.engine.send(Command::AddMagnet {
                magnet_or_url: trimmed.to_string(),
                save_path: None,
            });
            return;
        }

        // Обычный путь к файлу либо file:// URI (так ОС иногда передаёт путь
        // при "Открыть с помощью…").
        let path = if let Some(rest) = trimmed.strip_prefix("file://") {
            let mut decoded = urlencoding::decode(rest)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| rest.to_string());
            // file:///C:/foo.torrent -> /C:/foo.torrent -> C:/foo.torrent
            if cfg!(windows) && decoded.starts_with('/') && decoded.as_bytes().get(2) == Some(&b':') {
                decoded.remove(0);
            }
            PathBuf::from(decoded)
        } else {
            PathBuf::from(trimmed)
        };

        self.engine.send(Command::AddTorrentFile { path, save_path: None });
    }
}

fn default_download_dir() -> PathBuf {
    if let Some(dirs) = directories::UserDirs::new() {
        if let Some(downloads) = dirs.download_dir() {
            return downloads.join("TorrentBox");
        }
        return dirs.home_dir().join("TorrentBox");
    }
    PathBuf::from("TorrentBox")
}

impl eframe::App for TorrentBoxApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let prefs = Prefs {
            dark_mode: self.theme_mode == ThemeMode::Dark,
            download_dir: self.download_dir.clone(),
        };
        eframe::set_value(storage, STORAGE_KEY, &prefs);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            theme::apply(ctx, self.theme_mode);
            self.theme_applied = true;
        }

        // Магнет-ссылки/.torrent файлы, открытые через ОС (или присланные
        // повторным запуском TorrentBox, см. main.rs) — добавляем их сразу.
        let mut received_via_ipc = false;
        while let Ok(msg) = self.ipc_rx.try_recv() {
            self.open_uri_or_path(&msg);
            received_via_ipc = true;
        }
        if received_via_ipc {
            // Если окно было свёрнуто/в фоне — поднимаем его на передний план.
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Живое обновление списка: движок сам публикует снимок раз в ~0.5с,
        // а мы просто просим egui перерисовываться с той же частотой.
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        let snapshot = self.engine.snapshot();
        self.settings_state.listen_port = snapshot.listen_port;

        // --- Собственная шапка окна (вместо системной) -----------------------
        egui::TopBottomPanel::top("titlebar")
            .frame(egui::Frame::new().fill(theme::ACCENT).inner_margin(egui::Margin::same(0)))
            .exact_height(ui::titlebar::HEIGHT)
            .show(ctx, |ui| {
                ui::titlebar::show(ui);
            });

        // --- Верхняя панель -------------------------------------------------
        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::new().fill(ctx.style().visuals.panel_fill).inner_margin(egui::Margin::symmetric(14, 10)))
            .show(ctx, |ui| {
                if let Some(action) = ui::toolbar::show(ui, &mut self.search) {
                    self.handle_toolbar_action(action);
                }
            });

        // --- Левая панель фильтров ------------------------------------------
        egui::SidePanel::left("filters")
            .resizable(false)
            .default_width(170.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                for f in Filter::ALL {
                    let count = snapshot
                        .torrents
                        .iter()
                        .filter(|r| matches_filter_count(r, f))
                        .count();
                    let label = format!("{}  ({count})", f.label());
                    if ui.selectable_label(self.filter == f, label).clicked() {
                        self.filter = f;
                    }
                }
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(egui::RichText::new(format!("Всего торрентов: {}", snapshot.torrents.len())).small().color(egui::Color32::GRAY));
                if let Some(port) = snapshot.listen_port {
                    ui.label(egui::RichText::new(format!("Порт: {port}")).small().color(egui::Color32::GRAY));
                }
                if let Some(err) = &snapshot.last_error {
                    ui.add_space(8.0);
                    ui.colored_label(theme::DANGER, err);
                }
            });

        // --- Правая панель с деталями (если выбран торрент) ------------------
        if let Some(selected_id) = self.selected {
            if let Some(row) = snapshot.torrents.iter().find(|r| r.id == selected_id) {
                let row = row.clone();
                egui::SidePanel::right("details")
                    .resizable(true)
                    .default_width(340.0)
                    .min_width(280.0)
                    .show(ctx, |ui| {
                        if let Some(action) = ui::details::show(ui, &row) {
                            self.handle_details_action(action);
                        }
                    });
            } else {
                self.selected = None;
            }
        }

        // --- Центр: список торрентов -----------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if !snapshot.ready {
                ui.centered_and_justified(|ui| {
                    ui.label("Запуск torrent-движка…");
                });
                return;
            }
            if let Some(action) =
                ui::torrent_list::show(ui, &snapshot.torrents, self.filter, &self.search, self.selected)
            {
                self.handle_list_action(action);
            }
        });

        // --- Модальные окна ----------------------------------------------------
        if self.show_add_dialog {
            let mut open = self.show_add_dialog;
            if let Some(result) = ui::add_dialog::show(ctx, &mut open, &mut self.add_dialog_state) {
                match result {
                    AddDialogResult::AddMagnet { url, save_path } => {
                        self.engine.send(Command::AddMagnet { magnet_or_url: url, save_path: Some(save_path) });
                        self.show_add_dialog = false;
                        self.add_dialog_state = AddDialogState::new(self.download_dir.clone());
                    }
                    AddDialogResult::AddFile { path, save_path } => {
                        self.engine.send(Command::AddTorrentFile { path, save_path: Some(save_path) });
                        self.show_add_dialog = false;
                        self.add_dialog_state = AddDialogState::new(self.download_dir.clone());
                    }
                    AddDialogResult::Cancel => {
                        self.show_add_dialog = false;
                    }
                }
            } else {
                self.show_add_dialog = open;
            }
        }

        if self.show_settings {
            let mut open = self.show_settings;
            if let Some(result) = ui::settings::show(ctx, &mut open, &mut self.settings_state) {
                match result {
                    SettingsResult::Save { download_dir, dark_mode } => {
                        self.download_dir = download_dir;
                        let new_mode = if dark_mode { ThemeMode::Dark } else { ThemeMode::Light };
                        if new_mode != self.theme_mode {
                            self.theme_mode = new_mode;
                            theme::apply(ctx, self.theme_mode);
                        }
                        self.show_settings = false;
                    }
                    SettingsResult::Close => {
                        self.show_settings = false;
                    }
                }
            } else {
                self.show_settings = open;
            }
        }

        // Ручка изменения размера окна (см. src/ui/titlebar.rs) — рисуем
        // поверх всего остального в самом конце кадра.
        ui::titlebar::resize_grip(ctx);
    }
}

fn matches_filter_count(row: &TorrentRow, filter: Filter) -> bool {
    use crate::models::TorrentStatus::*;
    match filter {
        Filter::All => true,
        Filter::Downloading => matches!(row.status, Downloading | Checking | Connecting),
        Filter::Seeding => matches!(row.status, Seeding),
        Filter::Finished => matches!(row.status, Finished),
        Filter::Paused => matches!(row.status, Paused),
    }
}

impl TorrentBoxApp {
    fn handle_toolbar_action(&mut self, action: ToolbarAction) {
        match action {
            ToolbarAction::AddMagnet => {
                self.add_dialog_state = AddDialogState::new(self.download_dir.clone());
                self.add_dialog_state.tab = ui::AddTab::Magnet;
                self.show_add_dialog = true;
            }
            ToolbarAction::AddFile => {
                self.add_dialog_state = AddDialogState::new(self.download_dir.clone());
                self.add_dialog_state.tab = ui::AddTab::File;
                self.show_add_dialog = true;
            }
            ToolbarAction::PauseAll => self.engine.send(Command::PauseAll),
            ToolbarAction::ResumeAll => self.engine.send(Command::ResumeAll),
            ToolbarAction::OpenSettings => {
                self.settings_state.download_dir = self.download_dir.clone();
                self.settings_state.dark_mode = self.theme_mode == ThemeMode::Dark;
                self.show_settings = true;
            }
        }
    }

    fn handle_list_action(&mut self, action: ListAction) {
        match action {
            ListAction::Select(id) => self.selected = Some(id),
            ListAction::TogglePause(id) => {
                let snapshot = self.engine.snapshot();
                if let Some(row) = snapshot.torrents.iter().find(|r| r.id == id) {
                    if row.status == crate::models::TorrentStatus::Paused {
                        self.engine.send(Command::Resume(id));
                    } else {
                        self.engine.send(Command::Pause(id));
                    }
                }
            }
        }
    }

    fn handle_details_action(&mut self, action: DetailsAction) {
        match action {
            DetailsAction::TogglePause(id) => {
                let snapshot = self.engine.snapshot();
                if let Some(row) = snapshot.torrents.iter().find(|r| r.id == id) {
                    if row.status == crate::models::TorrentStatus::Paused {
                        self.engine.send(Command::Resume(id));
                    } else {
                        self.engine.send(Command::Pause(id));
                    }
                }
            }
            DetailsAction::RemoveKeepFiles(id) => {
                self.engine.send(Command::Remove { id, delete_files: false });
                self.selected = None;
            }
            DetailsAction::RemoveDeleteFiles(id) => {
                self.engine.send(Command::Remove { id, delete_files: true });
                self.selected = None;
            }
            DetailsAction::OpenFolder(path) => {
                let _ = open::that(path);
            }
            DetailsAction::Close => {
                self.selected = None;
            }
        }
    }
}
