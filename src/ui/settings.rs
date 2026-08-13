use eframe::egui::{self, RichText};

use super::{SettingsResult, SettingsState};
use crate::theme;

pub fn show(ctx: &egui::Context, open: &mut bool, state: &mut SettingsState) -> Option<SettingsResult> {
    let mut result = None;

    egui::Window::new("Настройки")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(RichText::new("Загрузка").strong());
            ui.horizontal(|ui| {
                ui.label(state.download_dir.display().to_string());
                if ui.button("Изменить…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        state.download_dir = dir;
                    }
                }
            });

            ui.add_space(10.0);
            if let Some(port) = state.listen_port {
                ui.label(format!("Порт для входящих соединений: {port}"));
            } else {
                ui.label("Порт для входящих соединений: определяется…");
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(RichText::new("Внешний вид").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(!state.dark_mode, "☀ Светлая").clicked() {
                    state.dark_mode = false;
                }
                if ui.selectable_label(state.dark_mode, "🌙 Тёмная").clicked() {
                    state.dark_mode = true;
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(RichText::new("Файлы по умолчанию").strong());
            ui.label(
                RichText::new("Открывать .torrent файлы и magnet-ссылки в TorrentBox")
                    .color(egui::Color32::GRAY)
                    .small(),
            );
            ui.add_space(4.0);
            if ui.button("🔗 Сделать TorrentBox приложением по умолчанию").clicked() {
                state.register_message = Some(match crate::register::register() {
                    Ok(()) => "Готово! .torrent-файлы и magnet-ссылки теперь открываются в TorrentBox.".to_string(),
                    Err(e) => format!("Не удалось: {e}"),
                });
            }
            if let Some(msg) = &state.register_message {
                ui.label(RichText::new(msg).small());
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Сохранить").color(egui::Color32::WHITE))
                                .fill(theme::ACCENT),
                        )
                        .clicked()
                    {
                        result = Some(SettingsResult::Save {
                            download_dir: state.download_dir.clone(),
                            dark_mode: state.dark_mode,
                        });
                    }
                    if ui.button("Закрыть").clicked() {
                        result = Some(SettingsResult::Close);
                    }
                });
            });
        });

    result
}
