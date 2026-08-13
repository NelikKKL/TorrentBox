use eframe::egui::{self, RichText};

use super::{AddDialogResult, AddDialogState, AddTab};
use crate::theme;

pub fn show(ctx: &egui::Context, open: &mut bool, state: &mut AddDialogState) -> Option<AddDialogResult> {
    let mut result = None;

    egui::Window::new("Добавить торрент")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(state.tab == AddTab::Magnet, "🔗 Магнет-ссылка")
                    .clicked()
                {
                    state.tab = AddTab::Magnet;
                }
                if ui
                    .selectable_label(state.tab == AddTab::File, "📄 .torrent файл")
                    .clicked()
                {
                    state.tab = AddTab::File;
                }
            });
            ui.separator();
            ui.add_space(6.0);

            match state.tab {
                AddTab::Magnet => {
                    ui.label("Вставьте magnet-ссылку или прямую ссылку на .torrent:");
                    ui.add(
                        egui::TextEdit::multiline(&mut state.magnet_text)
                            .desired_rows(3)
                            .hint_text("magnet:?xt=urn:btih:…"),
                    );
                }
                AddTab::File => {
                    ui.label("Выберите .torrent файл на диске:");
                    ui.horizontal(|ui| {
                        let text = state
                            .file_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "Файл не выбран".to_string());
                        ui.label(text);
                        if ui.button("Обзор…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Torrent", &["torrent"])
                                .pick_file()
                            {
                                state.file_path = Some(path);
                            }
                        }
                    });
                }
            }

            ui.add_space(10.0);
            ui.label("Папка для загрузки:");
            ui.horizontal(|ui| {
                ui.label(state.save_dir.display().to_string());
                if ui.button("Изменить…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        state.save_dir = dir;
                    }
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_add = match state.tab {
                        AddTab::Magnet => !state.magnet_text.trim().is_empty(),
                        AddTab::File => state.file_path.is_some(),
                    };

                    if ui
                        .add_enabled(
                            can_add,
                            egui::Button::new(RichText::new("Добавить").color(egui::Color32::WHITE))
                                .fill(theme::ACCENT),
                        )
                        .clicked()
                    {
                        result = Some(match state.tab {
                            AddTab::Magnet => AddDialogResult::AddMagnet {
                                url: state.magnet_text.trim().to_string(),
                                save_path: state.save_dir.clone(),
                            },
                            AddTab::File => AddDialogResult::AddFile {
                                path: state.file_path.clone().unwrap(),
                                save_path: state.save_dir.clone(),
                            },
                        });
                    }
                    if ui.button("Отмена").clicked() {
                        result = Some(AddDialogResult::Cancel);
                    }
                });
            });
        });

    result
}
