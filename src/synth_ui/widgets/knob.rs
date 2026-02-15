use eframe::egui::{self, pos2, vec2, Pos2, Response, Sense, Stroke, Ui};
use std::f32::consts::PI;

use crate::synth_ui::theme::SynthTheme;

#[allow(dead_code)]
const KNOB_RADIUS: f32 = 24.0;
const ARC_START: f32 = 225.0_f32 * PI / 180.0; // -135 deg from top (in painter coords)
const ARC_RANGE: f32 = 270.0_f32 * PI / 180.0;
const ARC_POINTS: usize = 50;

#[allow(dead_code)]
pub fn knob(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32, unit: &str) -> Response {
    let desired_size = vec2(58.0, 78.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

    let center = pos2(rect.center().x, rect.top() + KNOB_RADIUS + 4.0);
    let painter = ui.painter_at(rect);

    // Handle drag
    if response.dragged() {
        let delta = -response.drag_delta().y;
        let range = max - min;
        let speed = range / 200.0;
        *value = (*value + delta * speed).clamp(min, max);
    }

    // Normalize value to 0..1
    let range = max - min;
    let t = if range.abs() < f32::EPSILON { 0.0 } else { (*value - min) / range };

    // Background circle
    painter.circle_filled(center, KNOB_RADIUS, SynthTheme::KNOB_BG);
    painter.circle_stroke(center, KNOB_RADIUS, Stroke::new(1.5, SynthTheme::BORDER));

    // Background arc (track)
    draw_arc(&painter, center, KNOB_RADIUS - 4.0, 0.0, 1.0, Stroke::new(3.0, SynthTheme::BORDER));

    // Value arc
    if t > 0.001 {
        draw_arc(&painter, center, KNOB_RADIUS - 4.0, 0.0, t, Stroke::new(3.0, SynthTheme::ACCENT));
    }

    // Indicator line
    let angle = ARC_START - t * ARC_RANGE;
    let inner = pos2(
        center.x + (KNOB_RADIUS - 12.0) * angle.cos(),
        center.y - (KNOB_RADIUS - 12.0) * angle.sin(),
    );
    let outer = pos2(
        center.x + (KNOB_RADIUS - 5.0) * angle.cos(),
        center.y - (KNOB_RADIUS - 5.0) * angle.sin(),
    );
    painter.line_segment([inner, outer], Stroke::new(2.0, SynthTheme::TEXT));

    // Center dot
    painter.circle_filled(center, 3.0, SynthTheme::ACCENT);

    // Label below knob
    let label_pos = pos2(rect.center().x, center.y + KNOB_RADIUS + 4.0);
    painter.text(
        label_pos,
        egui::Align2::CENTER_TOP,
        label,
        egui::FontId::proportional(10.0),
        SynthTheme::TEXT_DIM,
    );

    // Value text
    let value_text = if unit.is_empty() {
        format!("{:.2}", *value)
    } else {
        format!("{:.1}{}", *value, unit)
    };
    let value_pos = pos2(rect.center().x, center.y + KNOB_RADIUS + 16.0);
    painter.text(
        value_pos,
        egui::Align2::CENTER_TOP,
        value_text,
        egui::FontId::proportional(9.0),
        SynthTheme::TEXT,
    );

    response
}

/// Compact knob for channel strips (36×56px).
pub fn mini_knob(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> Response {
    let r = 16.0;
    let desired_size = vec2(36.0, 52.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

    let center = pos2(rect.center().x, rect.top() + r + 2.0);
    let painter = ui.painter_at(rect);

    if response.dragged() {
        let delta = -response.drag_delta().y;
        let range = max - min;
        let speed = range / 150.0;
        *value = (*value + delta * speed).clamp(min, max);
    }

    let range = max - min;
    let t = if range.abs() < f32::EPSILON { 0.0 } else { (*value - min) / range };

    painter.circle_filled(center, r, SynthTheme::KNOB_BG);
    painter.circle_stroke(center, r, Stroke::new(1.0, SynthTheme::BORDER));

    draw_arc(&painter, center, r - 3.0, 0.0, 1.0, Stroke::new(2.0, SynthTheme::BORDER));
    if t > 0.001 {
        draw_arc(&painter, center, r - 3.0, 0.0, t, Stroke::new(2.0, SynthTheme::ACCENT));
    }

    let angle = ARC_START - t * ARC_RANGE;
    let inner = pos2(
        center.x + (r - 8.0) * angle.cos(),
        center.y - (r - 8.0) * angle.sin(),
    );
    let outer = pos2(
        center.x + (r - 3.0) * angle.cos(),
        center.y - (r - 3.0) * angle.sin(),
    );
    painter.line_segment([inner, outer], Stroke::new(1.5, SynthTheme::TEXT));
    painter.circle_filled(center, 2.0, SynthTheme::ACCENT);

    let label_pos = pos2(rect.center().x, center.y + r + 2.0);
    painter.text(
        label_pos,
        egui::Align2::CENTER_TOP,
        label,
        egui::FontId::proportional(8.0),
        SynthTheme::TEXT_DIM,
    );

    let value_pos = pos2(rect.center().x, center.y + r + 12.0);
    painter.text(
        value_pos,
        egui::Align2::CENTER_TOP,
        format!("{:.2}", *value),
        egui::FontId::proportional(7.0),
        SynthTheme::TEXT,
    );

    response
}

fn draw_arc(painter: &egui::Painter, center: Pos2, radius: f32, from_t: f32, to_t: f32, stroke: Stroke) {
    let start_angle = ARC_START - from_t * ARC_RANGE;
    let end_angle = ARC_START - to_t * ARC_RANGE;
    let steps = ((ARC_POINTS as f32 * (to_t - from_t).abs()) as usize).max(2);
    let points: Vec<Pos2> = (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let angle = start_angle + (end_angle - start_angle) * t;
            pos2(center.x + radius * angle.cos(), center.y - radius * angle.sin())
        })
        .collect();

    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
}
