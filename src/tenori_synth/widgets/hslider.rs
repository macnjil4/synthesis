use eframe::egui;
use eframe::egui::Stroke;

use crate::tenori_synth::theme::Theme;

pub fn hslider(ui: &mut egui::Ui, label: &str, value: &mut f32, min: f32, max: f32) {
    let desired_height = 14.0;
    let available_width = ui.available_width();

    ui.horizontal(|ui| {
        // Label
        ui.allocate_ui(egui::vec2(42.0, desired_height), |ui| {
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(9.0)
                            .strong()
                            .color(Theme::TEXT),
                    );
                },
            );
        });

        // Track
        let track_width = (available_width - 42.0 - 30.0 - 16.0).max(40.0);
        let (track_rect, response) = ui.allocate_exact_size(
            egui::vec2(track_width, desired_height),
            egui::Sense::click_and_drag(),
        );

        if (response.clicked() || response.dragged())
            && let Some(pos) = response.interact_pointer_pos()
        {
            let pct = ((pos.x - track_rect.left()) / track_rect.width()).clamp(0.0, 1.0);
            *value = min + pct * (max - min);
        }

        // Draw track
        let painter = ui.painter_at(track_rect);
        let track_y = track_rect.center().y;
        let track_h = 6.0;
        let track_draw_rect = egui::Rect::from_center_size(
            egui::pos2(track_rect.center().x, track_y),
            egui::vec2(track_rect.width(), track_h),
        );

        // Background track
        painter.rect(
            track_draw_rect,
            3.0,
            Theme::KNOB_BG,
            Stroke::new(1.0, Theme::BORDER),
            egui::StrokeKind::Inside,
        );

        // Fill
        let range = max - min;
        let pct = if range.abs() < f32::EPSILON {
            0.0
        } else {
            ((*value - min) / range).clamp(0.0, 1.0)
        };
        let fill_width = pct * track_draw_rect.width();
        if fill_width > 0.5 {
            let fill_rect = egui::Rect::from_min_size(
                track_draw_rect.min,
                egui::vec2(fill_width, track_h),
            );
            painter.rect_filled(fill_rect, 3.0, Theme::ACCENT_DARK);
        }

        // Thumb
        let thumb_x = track_draw_rect.left() + pct * track_draw_rect.width();
        let thumb_rect = egui::Rect::from_center_size(
            egui::pos2(thumb_x, track_y),
            egui::vec2(10.0, 12.0),
        );
        painter.rect(
            thumb_rect,
            2.0,
            Theme::PANEL_LIGHT,
            Stroke::new(1.0, Theme::ACCENT),
            egui::StrokeKind::Inside,
        );

        // Value
        ui.allocate_ui(egui::vec2(24.0, desired_height), |ui| {
            ui.label(
                egui::RichText::new(format!("{}", *value as i32))
                    .size(9.0)
                    .color(Theme::TEXT_DIM),
            );
        });
    });

    ui.add_space(2.0);
}
