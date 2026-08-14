//! Фирменная тема TorrentBox.
//!
//! Основной акцентный цвет — #483524 (тёмный "шоколадно-коричневый").
//! Тема собрана в двух вариантах — светлом и тёмном, как в оригинальном
//! LibreTorrent.

use eframe::egui::{self, Color32, CornerRadius, Stroke};

/// Основной акцентный цвет бренда — #483524.
pub const ACCENT: Color32 = Color32::from_rgb(0x48, 0x35, 0x24);
/// Более светлый оттенок акцента (наведение / активные элементы).
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x76, 0x68, 0x5B);
/// Более тёмный оттенок акцента (нажатие).
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x38, 0x29, 0x1C);

pub const SUCCESS: Color32 = Color32::from_rgb(0x4C, 0xA6, 0x5B); // раздаётся / завершено
pub const WARNING: Color32 = Color32::from_rgb(0xD9, 0xA5, 0x30); // проверка / ожидание
pub const DANGER: Color32 = Color32::from_rgb(0xC1, 0x4B, 0x3F); // ошибка

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    // Явно фиксируем тему (Light/Dark) в самом egui. Начиная с egui 0.31
    // у контекста появилось собственное понятие "предпочтения темы"
    // (ThemePreference::System/Light/Dark), которое по умолчанию стоит
    // в System и следует за тёмной/светлой темой ОС. Если его не задать
    // явно через ctx.set_theme(...), то ctx.set_visuals(...) ниже будет
    // писать цвета не в тот "слот" темы, который реально показывается —
    // из-за этого переключатель в Настройках визуально не работает
    // (особенно у пользователей с тёмной темой ОС по умолчанию).
    let egui_theme = match mode {
        ThemeMode::Light => egui::Theme::Light,
        ThemeMode::Dark => egui::Theme::Dark,
    };
    ctx.set_theme(egui_theme);

    let mut visuals = match mode {
        ThemeMode::Light => egui::Visuals::light(),
        ThemeMode::Dark => egui::Visuals::dark(),
    };

    let (panel_bg, window_bg, faint_bg, extreme_bg) = match mode {
        ThemeMode::Light => (
            Color32::from_rgb(0xFB, 0xF7, 0xF2),
            Color32::from_rgb(0xFF, 0xFF, 0xFF),
            Color32::from_rgb(0xF1, 0xE9, 0xDF),
            Color32::from_rgb(0xFF, 0xFF, 0xFF),
        ),
        ThemeMode::Dark => (
            Color32::from_rgb(0x1E, 0x1A, 0x16),
            Color32::from_rgb(0x27, 0x22, 0x1D),
            Color32::from_rgb(0x33, 0x2B, 0x23),
            Color32::from_rgb(0x16, 0x13, 0x10),
        ),
    };

    visuals.override_text_color = None;
    visuals.panel_fill = panel_bg;
    visuals.window_fill = window_bg;
    visuals.faint_bg_color = faint_bg;
    visuals.extreme_bg_color = extreme_bg;
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.55);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.hyperlink_color = ACCENT_DARK;

    visuals.widgets.hovered.weak_bg_fill = ACCENT_LIGHT.linear_multiply(0.35);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_LIGHT);
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DARK);
    visuals.widgets.inactive.weak_bg_fill = faint_bg;

    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(8);
    }
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(8);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

/// Цвет статуса торрента для индикаторов и полос прогресса.
pub fn status_color(status: crate::models::TorrentStatus) -> Color32 {
    use crate::models::TorrentStatus::*;
    match status {
        Downloading => ACCENT,
        Seeding | Finished => SUCCESS,
        Paused => Color32::GRAY,
        Checking | Connecting => WARNING,
        Error => DANGER,
    }
}
