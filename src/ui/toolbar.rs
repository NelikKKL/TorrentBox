use eframe::egui::{self, RichText};

use super::ToolbarAction;
use crate::theme;

pub fn show(ui: &mut egui::Ui, search: &mut String) -> Option<ToolbarAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("📦 TorrentBox").heading().color(theme::ACCENT_DARK));
        ui.add_space(16.0);

        let search_field = egui::TextEdit::singleline(search)
            .hint_text("Поиск по названию…")
            .desired_width(260.0);
        ui.add(search_field);

        if !search.is_empty() && ui.small_button("✕").clicked() {
            search.clear();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("⚙")
                .on_hover_text("Настройки")
                .clicked()
            {
                action = Some(ToolbarAction::OpenSettings);
            }
            ui.separator();
            if ui.button("▶ Все").on_hover_text("Возобновить все").clicked() {
                action = Some(ToolbarAction::ResumeAll);
            }
            if ui.button("⏸ Все").on_hover_text("Приостановить все").clicked() {
                action = Some(ToolbarAction::PauseAll);
            }
            ui.separator();
            if ui
                .button(RichText::new("+ Файл").strong())
                .on_hover_text("Добавить .torrent файл")
                .clicked()
            {
                action = Some(ToolbarAction::AddFile);
            }
            if ui
                .add(egui::Button::new(RichText::new("+ Магнет-ссылка").strong().color(egui::Color32::WHITE))
                    .fill(theme::ACCENT))
                .clicked()
            {
                action = Some(ToolbarAction::AddMagnet);
            }
        });
    });

    action
}
