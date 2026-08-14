//! Своя шапка окна вместо системной.
//!
//! Мы отключаем нативное оформление окна (`with_decorations(false)` в
//! main.rs) и рисуем шапку сами — поэтому её можно покрасить в акцентный
//! цвет темы. Работает одинаково на Windows и на Linux (X11 — надёжно;
//! на некоторых Wayland-компоузиторах перетаскивание окна за пустую часть
//! шапки может не сработать — это ограничение самого протокола Wayland,
//! а не наше; в таком случае окно всё ещё можно двигать через Alt+ЛКМ,
//! как и любое другое окно без рамки).

use eframe::egui::{self, vec2, Align2, Color32, CursorIcon, FontId, Id, PointerButton, RichText, Sense};

use crate::theme;

pub const HEIGHT: f32 = 38.0;

/// Рисует содержимое шапки. Вызывать внутри `TopBottomPanel::top(...).show(ctx, |ui| ...)`.
pub fn show(ui: &mut egui::Ui) {
    let rect = ui.max_rect();

    // Вся шапка целиком реагирует на перетаскивание/двойной клик...
    let drag_response = ui.interact(rect, Id::new("titlebar_drag_area"), Sense::click_and_drag());
    if drag_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }
    if drag_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    // ...а поверх неё рисуются иконка/название и кнопки — они добавляются
    // позже, поэтому именно они получают клики в своей области, а не
    // область перетаскивания под ними.
    ui.horizontal_centered(|ui| {
        ui.add_space(10.0);
        ui.label(RichText::new("📦").size(16.0));
        ui.label(
            RichText::new("TorrentBox")
                .strong()
                .size(14.5)
                .color(Color32::WHITE),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(6.0);
            window_buttons(ui);
        });
    });

    // Тонкая линия снизу шапки, чтобы визуально отделить её от остального окна.
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0, theme::ACCENT_DARK),
    );
}

fn window_buttons(ui: &mut egui::Ui) {
    let btn_size = vec2(30.0, HEIGHT);
    let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, 230);

    let close = ui.add_sized(btn_size, egui::Button::new(RichText::new("✕").size(14.0).color(text_color)));
    if close.on_hover_text("Закрыть").clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let (icon, tip) = if is_maximized { ("🗗", "Восстановить") } else { ("🗖", "Развернуть") };
    let maximize = ui.add_sized(btn_size, egui::Button::new(RichText::new(icon).size(13.0).color(text_color)));
    if maximize.on_hover_text(tip).clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }

    let minimize = ui.add_sized(btn_size, egui::Button::new(RichText::new("🗕").size(13.0).color(text_color)));
    if minimize.on_hover_text("Свернуть").clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }
}

/// Небольшая "ручка" изменения размера в правом нижнem углу окна — нужна,
/// т.к. вместе с системным оформлением пропадают и системные рамки для
/// изменения размера. Тянуть можно и за неё, а разворачивать/восстанавливать —
/// кнопкой в шапке или двойным кликом по шапке.
pub fn resize_grip(ctx: &egui::Context) {
    use egui::viewport::ResizeDirection;

    let screen = ctx.screen_rect();
    let size = 14.0;
    let rect = egui::Rect::from_min_size(screen.right_bottom() - vec2(size, size), vec2(size, size));

    egui::Area::new(Id::new("torrentbox_resize_grip"))
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let response = ui.interact(rect, Id::new("torrentbox_resize_grip_hit"), Sense::drag());
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if response.drag_started_by(PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(ResizeDirection::SouthEast));
            }
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "⋰",
                FontId::proportional(12.0),
                ui.visuals().weak_text_color(),
            );
        });
}
