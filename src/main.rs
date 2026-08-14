// Консоль на Windows не должна открываться никогда — ни в debug, ни в release.
#![windows_subsystem = "windows"]

mod app;
mod engine;
mod models;
mod register;
mod theme;
mod ui;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use app::TorrentBoxApp;

/// Локальный порт для связи между экземплярами TorrentBox (127.0.0.1 —
/// значит, снаружи компьютера недоступен). Используется, чтобы клик по
/// magnet-ссылке в браузере или двойной клик по .torrent файлу, когда
/// TorrentBox уже запущен, не открывал второе окно, а просто добавлял
/// торрент в уже работающее.
const IPC_PORT: u16 = 58973;

fn main() -> eframe::Result<()> {
    // Аргументы командной строки: путь к .torrent, magnet-ссылка, либо и то,
    // и другое сразу (ОС передаёт их так при "Открыть с помощью…").
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (ipc_tx, ipc_rx) = mpsc::channel::<String>();

    match TcpListener::bind(("127.0.0.1", IPC_PORT)) {
        Ok(listener) => {
            // Мы — первый (главный) экземпляр. Собственные аргументы командной
            // строки обрабатываем точно так же, как и то, что придёт по IPC.
            for a in &args {
                let _ = ipc_tx.send(a.clone());
            }

            let tx_for_thread = ipc_tx.clone();
            std::thread::Builder::new()
                .name("torrentbox-ipc".into())
                .spawn(move || {
                    for stream in listener.incoming().flatten() {
                        for line in BufReader::new(stream).lines().flatten() {
                            let _ = tx_for_thread.send(line);
                        }
                    }
                })
                .ok();

            run_gui(ipc_rx)
        }
        Err(_) => {
            // Порт занят — значит, TorrentBox уже запущен. Просто передаём
            // ему наши аргументы (если есть) через сокет и сразу выходим,
            // не открывая второе окно.
            if !args.is_empty() {
                if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", IPC_PORT)) {
                    for a in &args {
                        let _ = writeln!(stream, "{a}");
                    }
                }
            }
            Ok(())
        }
    }
}

fn run_gui(ipc_rx: mpsc::Receiver<String>) -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("встроенная иконка assets/icon.png повреждена");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("TorrentBox")
            .with_inner_size([1080.0, 680.0])
            .with_min_inner_size([760.0, 480.0])
            // Убираем системную рамку/шапку — свою (в цвете темы) рисуем
            // сами в src/ui/titlebar.rs. Работает на Windows и Linux.
            .with_decorations(false)
            .with_icon(icon),
        persist_window: true,
        // Без этого eframe пытается сам подстроиться под системную
        // светлую/тёмную тему и на каждом кадре переопределяет наши
        // цвета — тогда переключатель темы в Настройках визуально не
        // будет работать. Управляем темой полностью сами (src/theme.rs).
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "torrentbox",
        native_options,
        Box::new(move |cc| Ok(Box::new(TorrentBoxApp::new(cc, ipc_rx)))),
    )
}
