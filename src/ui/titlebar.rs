//! Своя шапка окна вместо системной.
//!
//! Мы отключаем нативное оформление окна (`with_decorations(false)` в
//! main.rs) и рисуем шапку сами — поэтому её можно покрасить в акцентный
//! цвет темы. Кнопки — обычным текстом (не эмодзи-иконками): у эмодзи вроде
//! 🗗/🗖/🗕 нет гарантированного глифа во встроенном шрифте egui на всех
//! системах, из-за чего они могли не отрисовываться.
//!
//! Известный нюанс безрамочных окон в egui (см. github.com/emilk/egui,
//! issues #1518 и #3669): если сначала сделать `interact()` на весь
//! прямоугольник шапки (для перетаскивания), а кнопки добавить поверх —
//! на некоторых системах (в частности GNOME) клик по кнопке вместо этого
//! начинает тащить окно. Поэтому здесь сделано наоборот: сначала рисуются
//! и опрашиваются кнопки, и только если ни одна из них не под курсором —
//! оставшаяся часть шапки становится областью перетаскивания.

use eframe::egui::{self, Color32, Id, PointerButton, RichText, Sense};

use crate::theme;

pub const HEIGHT: f32 = 38.0;

/// Рисует содержимое шапки. Вызывать внутри `TopBottomPanel::top(...).show(ctx, |ui| ...)`.
pub fn show(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let mut pointer_over_button = false;

    ui.horizontal_centered(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new("TorrentBox").strong().size(14.5).color(Color32::WHITE));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(6.0);
            pointer_over_button = window_buttons(ui);
        });
    });

    // Перетаскивание/двойной клик — только там, где нет кнопок (см. пояснение выше).
    if !pointer_over_button {
        let drag_response = ui.interact(rect, Id::new("titlebar_drag_area"), Sense::click_and_drag());
        if drag_response.double_clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
        } else if drag_response.drag_started_by(PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }

    // Тонкая линия снизу шапки, чтобы визуально отделить её от остального окна.
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0_f32, theme::ACCENT_DARK),
    );
}

/// Рисует кнопки "Закрыть" / "Развернуть" / "Свернуть" текстом.
/// Возвращает `true`, если курсор сейчас над одной из них (см. `show`).
fn window_buttons(ui: &mut egui::Ui) -> bool {
    let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, 235);
    let font_size = 12.5;

    let close = ui.add(
        egui::Button::new(RichText::new("Закрыть").size(font_size).color(text_color))
            .min_size(egui::vec2(0.0, HEIGHT)),
    );
    if close.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let maximize = ui.add(
        egui::Button::new(RichText::new("[ ]").size(font_size).color(text_color))
            .min_size(egui::vec2(0.0, HEIGHT)),
    );
    let maximize_hovered = maximize.hovered();
    let maximize_dragged = maximize.dragged();
    let maximize_tip = if is_maximized { "Восстановить" } else { "Развернуть" };
    if maximize.on_hover_text(maximize_tip).clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }

    let minimize = ui.add(
        egui::Button::new(RichText::new("Свернуть").size(font_size).color(text_color))
            .min_size(egui::vec2(0.0, HEIGHT)),
    );
    if minimize.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    close.hovered() || maximize_hovered || minimize.hovered()
        || close.dragged() || maximize_dragged || minimize.dragged()
}

/// Небольшая "ручка" изменения размера в правом нижнем углу окна — нужна,
/// т.к. вместе с системным оформлением пропадают и системные рамки для
/// изменения размера. Рисуется простыми линиями (не текстом/эмодзи), чтобы
/// не зависеть от того, какие глифы есть в шрифте.
pub fn resize_grip(ctx: &egui::Context) {
    use egui::viewport::ResizeDirection;

    let screen = ctx.screen_rect();
    let size = 16.0;
    let rect = egui::Rect::from_min_size(screen.right_bottom() - egui::vec2(size, size), egui::vec2(size, size));

    egui::Area::new(Id::new("torrentbox_resize_grip"))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let response = ui.interact(rect, Id::new("torrentbox_resize_grip_hit"), Sense::drag());
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
            }
            if response.drag_started_by(PointerButton::Primary) {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::BeginResize(ResizeDirection::SouthEast));
            }

            let color = ui.visuals().weak_text_color();
            let stroke = egui::Stroke::new(1.0_f32, color);
            let painter = ui.painter();
            // Три диагональные чёрточки — классический вид ручки resize в углу окна.
            for i in 0..3 {
                let offset = 4.0 + i as f32 * 4.0;
                painter.line_segment(
                    [
                        egui::pos2(rect.right() - offset, rect.bottom() - 2.0),
                        egui::pos2(rect.right() - 2.0, rect.bottom() - offset),
                    ],
                    stroke,
                );
            }
        });
}
