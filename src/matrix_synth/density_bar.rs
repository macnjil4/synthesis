use eframe::egui;
use eframe::egui::Stroke;

use super::state::{ChannelMode, MatrixState, COLS};
use super::theme::Theme;

pub fn draw(ui: &mut egui::Ui, state: &MatrixState) {
    ui.horizontal(|ui| {
        ui.add_space(Theme::NOTE_LABEL_WIDTH);

        for col in 0..COLS {
            let density = state.col_density(col);
            let is_active = state.is_playing && state.play_col == col as i32;

            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(Theme::CELL_SIZE, Theme::DENSITY_HEIGHT),
                egui::Sense::hover(),
            );

            let fill = if density > 0 {
                let alpha = ((density as f32 / 6.0).min(1.0) * 255.0) as u8;
                match state.mode {
                    ChannelMode::Lead => egui::Color32::from_rgba_premultiplied(234, 179, 8, alpha),
                    ChannelMode::Drummer => egui::Color32::from_rgba_premultiplied(124, 58, 237, alpha),
                    ChannelMode::Bass => egui::Color32::from_rgba_premultiplied(236, 72, 153, alpha),
                }
            } else {
                Theme::CELL_OFF
            };

            let border = if is_active {
                Theme::ACCENT_LIGHT
            } else {
                Theme::BORDER
            };

            let painter = ui.painter();
            painter.rect_filled(rect, 3.0, fill);
            painter.rect_stroke(rect, 3.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);

            ui.add_space(Theme::CELL_GAP);
        }
    });
}
