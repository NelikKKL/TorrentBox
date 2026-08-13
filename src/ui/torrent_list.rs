use eframe::egui::{self, Color32, RichText, Sense, Stroke};

use super::{Filter, ListAction};
use crate::models::{format_bytes, format_percent, TorrentRow, TorrentStatus};
use crate::theme;

pub fn show(
    ui: &mut egui::Ui,
    rows: &[TorrentRow],
    filter: Filter,
    search: &str,
    selected: Option<usize>,
) -> Option<ListAction> {
    let mut action = None;
    let search_lower = search.to_lowercase();

    let filtered: Vec<&TorrentRow> = rows
        .iter()
        .filter(|r| matches_filter(r.status, filter))
        .filter(|r| search_lower.is_empty() || r.name.to_lowercase().contains(&search_lower))
        .collect();

    if filtered.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("📭").size(40.0));
            ui.label(
                RichText::new(if rows.is_empty() {
                    "Пока нет торрентов — добавьте магнет-ссылку или .torrent файл"
                } else {
                    "Ничего не найдено"
                })
                .color(Color32::GRAY),
            );
        });
        return None;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in filtered {
            let is_selected = selected == Some(row.id);
            if let Some(a) = show_row(ui, row, is_selected) {
                action = Some(a);
            }
            ui.add_space(2.0);
        }
    });

    action
}

fn matches_filter(status: TorrentStatus, filter: Filter) -> bool {
    match filter {
        Filter::All => true,
        Filter::Downloading => matches!(status, TorrentStatus::Downloading | TorrentStatus::Checking | TorrentStatus::Connecting),
        Filter::Seeding => matches!(status, TorrentStatus::Seeding),
        Filter::Finished => matches!(status, TorrentStatus::Finished),
        Filter::Paused => matches!(status, TorrentStatus::Paused),
    }
}

fn show_row(ui: &mut egui::Ui, row: &TorrentRow, is_selected: bool) -> Option<ListAction> {
    let mut action = None;

    let frame = egui::Frame::new()
        .fill(if is_selected {
            theme::ACCENT.linear_multiply(0.12)
        } else {
            ui.visuals().panel_fill
        })
        .stroke(if is_selected {
            Stroke::new(1.5, theme::ACCENT)
        } else {
            Stroke::NONE
        })
        .corner_radius(10)
        .inner_margin(egui::Margin::symmetric(12, 10));

    let response = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Круглая кнопка пауза/старт слева, как в LibreTorrent.
                let (icon, hover) = match row.status {
                    TorrentStatus::Paused => ("▶", "Возобновить"),
                    TorrentStatus::Error => ("!", "Ошибка — нажмите для повтора"),
                    _ => ("⏸", "Приостановить"),
                };
                let btn = egui::Button::new(RichText::new(icon).size(16.0))
                    .fill(theme::status_color(row.status).linear_multiply(0.25))
                    .corner_radius(16)
                    .min_size(egui::vec2(32.0, 32.0));
                if ui.add(btn).on_hover_text(hover).clicked() {
                    action = Some(ListAction::TogglePause(row.id));
                }

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(&row.name).strong().size(15.0));

                    let bar = egui::ProgressBar::new(row.progress)
                        .fill(theme::status_color(row.status))
                        .desired_height(6.0)
                        .show_percentage();
                    ui.add(bar);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(row.status.label())
                                .color(theme::status_color(row.status))
                                .small(),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("↓ {}", row.download_speed))
                                .small()
                                .color(Color32::GRAY),
                        );
                        ui.label(
                            RichText::new(format!("↑ {}", row.upload_speed))
                                .small()
                                .color(Color32::GRAY),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "{} / {} · {} · осталось {}",
                                format_bytes(row.downloaded_bytes),
                                format_bytes(row.total_bytes),
                                format_percent(row.progress),
                                row.eta
                            ))
                            .small()
                            .color(Color32::GRAY),
                        );
                    });

                    if let Some(err) = &row.error {
                        ui.label(RichText::new(format!("Ошибка: {err}")).color(theme::DANGER).small());
                    }
                });
            });
        })
        .response;

    let response = response.interact(Sense::click());
    if response.clicked() {
        action = Some(ListAction::Select(row.id));
    }

    action
}
