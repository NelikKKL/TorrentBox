pub mod add_dialog;
pub mod details;
pub mod settings;
pub mod toolbar;
pub mod torrent_list;

use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    All,
    Downloading,
    Seeding,
    Finished,
    Paused,
}

impl Filter {
    pub const ALL: [Filter; 5] = [
        Filter::All,
        Filter::Downloading,
        Filter::Seeding,
        Filter::Finished,
        Filter::Paused,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Filter::All => "Все",
            Filter::Downloading => "Загружаются",
            Filter::Seeding => "Раздаются",
            Filter::Finished => "Завершены",
            Filter::Paused => "На паузе",
        }
    }
}

pub enum ToolbarAction {
    AddMagnet,
    AddFile,
    PauseAll,
    ResumeAll,
    OpenSettings,
}

pub enum ListAction {
    Select(usize),
    TogglePause(usize),
}

pub enum DetailsAction {
    TogglePause(usize),
    RemoveKeepFiles(usize),
    RemoveDeleteFiles(usize),
    OpenFolder(PathBuf),
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddTab {
    Magnet,
    File,
}

pub struct AddDialogState {
    pub tab: AddTab,
    pub magnet_text: String,
    pub file_path: Option<PathBuf>,
    pub save_dir: PathBuf,
}

impl AddDialogState {
    pub fn new(default_dir: PathBuf) -> Self {
        Self {
            tab: AddTab::Magnet,
            magnet_text: String::new(),
            file_path: None,
            save_dir: default_dir,
        }
    }
}

pub enum AddDialogResult {
    AddMagnet { url: String, save_path: PathBuf },
    AddFile { path: PathBuf, save_path: PathBuf },
    Cancel,
}

pub struct SettingsState {
    pub download_dir: PathBuf,
    pub dark_mode: bool,
    pub listen_port: Option<u16>,
    pub register_message: Option<String>,
}

pub enum SettingsResult {
    Save { download_dir: PathBuf, dark_mode: bool },
    Close,
}
