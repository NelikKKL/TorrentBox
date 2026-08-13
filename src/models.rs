//! Модели данных, которые движок (engine.rs) отдаёт интерфейсу.
//! Здесь нет прямой зависимости от librqbit — только простые,
//! удобные для отрисовки в egui структуры.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TorrentStatus {
    Checking,
    Connecting,
    Downloading,
    Seeding,
    Finished,
    Paused,
    Error,
}

impl TorrentStatus {
    pub fn label(&self) -> &'static str {
        match self {
            TorrentStatus::Checking => "Проверка файлов…",
            TorrentStatus::Connecting => "Подключение…",
            TorrentStatus::Downloading => "Загрузка…",
            TorrentStatus::Seeding => "Раздача…",
            TorrentStatus::Finished => "Завершено",
            TorrentStatus::Paused => "Пауза",
            TorrentStatus::Error => "Ошибка",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileRow {
    pub path: String,
    pub size_bytes: u64,
    pub progress: f32, // 0.0..=1.0
    pub wanted: bool,
}

#[derive(Clone, Debug)]
pub struct TorrentRow {
    /// Локальный числовой id внутри сессии librqbit (используется для команд).
    pub id: usize,
    pub name: String,
    pub info_hash: String,
    pub status: TorrentStatus,
    pub progress: f32, // 0.0..=1.0
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    /// Уже отформатированная librqbit строка вида "1.2 MiB/s" (см. engine.rs).
    pub download_speed: String,
    pub upload_speed: String,
    /// Уже отформатированная librqbit строка вида "5m 30s" / "∞".
    pub eta: String,
    pub save_path: PathBuf,
    pub error: Option<String>,
    pub files: Vec<FileRow>,
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["Б", "КБ", "МБ", "ГБ", "ТБ", "ПБ"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn format_percent(progress: f32) -> String {
    format!("{:.1}%", (progress * 100.0).clamp(0.0, 100.0))
}
