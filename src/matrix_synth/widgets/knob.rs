use eframe::egui;
use eframe::egui::Stroke;

use crate::matrix_synth::theme::Theme;

#[allow(dead_code)]
pub struct KnobResponse {
    pub changed: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn knob(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    default: f32,
    unit: &str,
    size: f32,
) -> KnobResponse {
    let total_height = size + 2.0 + 12.0 + 12.0;
    let total_width = size + 8.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(total_width, total_height),
        egui::Sense::click_and_drag(),
    );

    // Double-click -> reset
    if response.double_clicked() {
        *value = default;
    }

    // Vertical drag
    if response.dragged() {
        let delta = -response.drag_delta().y;
        let range = max - min;
        if range.abs() > f32::EPSILON {
            *value = (*value + delta * range / 150.0).clamp(min, max);
        }
    }

    // Render
    let painter = ui.painter_at(rect);
    let cx = rect.center_top() + egui::vec2(0.0, size / 2.0);
    let radius = size / 2.0 - 4.0;
    let range = max - min;
    let pct = if range.abs() < f32::EPSILON {
        0.0
    } else {
        ((*value - min) / range).clamp(0.0, 1.0)
    };

    // 1. Background circle
    painter.circle(
        cx,
        radius,
        Theme::KNOB_BG,
        Stroke::new(1.5, Theme::BORDER),
    );

    // 2. Arc of progression (270 degrees)
    let start_angle = -225.0_f32.to_radians(); // bottom-left
    let arc_angle = pct * 270.0_f32.to_radians();
    let arc_radius = radius - 2.0;
    let segments = 32;
    if pct > 0.001 {
        let points: Vec<egui::Pos2> = (0..=segments)
            .map(|i| {
                let t = i as f32 / segments as f32;
                let angle = start_angle + t * arc_angle;
                egui::pos2(
                    cx.x + arc_radius * angle.cos(),
                    cx.y - arc_radius * angle.sin(),
                )
            })
            .collect();
        for window in points.windows(2) {
            painter.line_segment(
                [window[0], window[1]],
                Stroke::new(2.5, Theme::ACCENT_DARK),
            );
        }
    }

    // 3. Indicator line
    let indicator_angle = (-135.0 + pct * 270.0).to_radians();
    let line_end = egui::pos2(
        cx.x + (radius - 2.0) * indicator_angle.cos(),
        cx.y - (radius - 2.0) * indicator_angle.sin(),
    );
    let line_start = egui::pos2(
        cx.x + 4.0 * indicator_angle.cos(),
        cx.y - 4.0 * indicator_angle.sin(),
    );
    painter.line_segment([line_start, line_end], Stroke::new(1.5, Theme::ACCENT_LIGHT));

    // 4. Center dot
    painter.circle_filled(cx, 3.0, Theme::ACCENT);

    // 5. Value text
    let value_text = if unit.is_empty() {
        format!("{}", *value as i32)
    } else {
        format!("{}{}", *value as i32, unit)
    };
    let value_y = cx.y + radius + 6.0;
    painter.text(
        egui::pos2(cx.x, value_y),
        egui::Align2::CENTER_CENTER,
        &value_text,
        egui::FontId::proportional(9.0),
        Theme::TEXT_DIM,
    );

    // 6. Label text
    let label_y = value_y + 12.0;
    painter.text(
        egui::pos2(cx.x, label_y),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::new(9.0, egui::FontFamily::Proportional),
        Theme::TEXT,
    );

    KnobResponse {
        changed: response.dragged() || response.double_clicked(),
    }
}
