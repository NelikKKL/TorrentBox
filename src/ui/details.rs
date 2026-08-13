use eframe::egui::{self, Color32, RichText};

use super::DetailsAction;
use crate::models::{format_bytes, format_percent, TorrentRow, TorrentStatus};
use crate::theme;

pub fn show(ui: &mut egui::Ui, row: &TorrentRow) -> Option<DetailsAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(RichText::new(&row.name).strong().size(16.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("✕").on_hover_text("Закрыть").clicked() {
                action = Some(DetailsAction::Close);
            }
        });
    });
    ui.add_space(6.0);

    ui.add(
        egui::ProgressBar::new(row.progress)
            .fill(theme::status_color(row.status))
            .desired_height(10.0)
            .show_percentage(),
    );
    ui.add_space(10.0);

    egui::Grid::new("details_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label(RichText::new("Статус").color(Color32::GRAY));
            ui.label(row.status.label());
            ui.end_row();

            ui.label(RichText::new("Размер").color(Color32::GRAY));
            ui.label(format!(
                "{} из {}",
                format_bytes(row.downloaded_bytes),
                format_bytes(row.total_bytes)
            ));
            ui.end_row();

            ui.label(RichText::new("Прогресс").color(Color32::GRAY));
            ui.label(format_percent(row.progress));
            ui.end_row();

            ui.label(RichText::new("Скорость загрузки").color(Color32::GRAY));
            ui.label(format!("↓ {}", row.download_speed));
            ui.end_row();

            ui.label(RichText::new("Скорость отдачи").color(Color32::GRAY));
            ui.label(format!("↑ {}", row.upload_speed));
            ui.end_row();

            ui.label(RichText::new("Роздано").color(Color32::GRAY));
            ui.label(format_bytes(row.uploaded_bytes));
            ui.end_row();

            ui.label(RichText::new("Осталось").color(Color32::GRAY));
            ui.label(&row.eta);
            ui.end_row();

            ui.label(RichText::new("Папка").color(Color32::GRAY));
            ui.label(row.save_path.display().to_string());
            ui.end_row();

            ui.label(RichText::new("Хэш").color(Color32::GRAY));
            ui.label(RichText::new(&row.info_hash).monospace().small());
            ui.end_row();
        });

    ui.add_space(14.0);
    ui.separator();
    ui.add_space(8.0);

    ui.horizontal_wrapped(|ui| {
        let toggle_label = if row.status == TorrentStatus::Paused {
            "▶ Возобновить"
        } else {
            "⏸ Пауза"
        };
        if ui.button(toggle_label).clicked() {
            action = Some(DetailsAction::TogglePause(row.id));
        }
        if ui.button("📂 Открыть папку").clicked() {
            action = Some(DetailsAction::OpenFolder(row.save_path.clone()));
        }
        if ui
            .add(egui::Button::new("Удалить").fill(Color32::TRANSPARENT))
            .clicked()
        {
            action = Some(DetailsAction::RemoveKeepFiles(row.id));
        }
        if ui
            .add(egui::Button::new(RichText::new("Удалить с файлами").color(Color32::WHITE)).fill(theme::DANGER))
            .clicked()
        {
            action = Some(DetailsAction::RemoveDeleteFiles(row.id));
        }
    });

    action
}
