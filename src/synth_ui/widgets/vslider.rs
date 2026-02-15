use eframe::egui::{self, pos2, vec2, Rect, Response, Sense, Stroke, Ui};

use crate::synth_ui::theme::SynthTheme;

const TRACK_WIDTH: f32 = 8.0;
const TRACK_HEIGHT: f32 = 90.0;
const THUMB_W: f32 = 16.0;
const THUMB_H: f32 = 12.0;

pub fn vslider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> Response {
    let desired_size = vec2(36.0, TRACK_HEIGHT + 30.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

    let painter = ui.painter_at(rect);
    let track_x = rect.center().x;
    let track_top = rect.top() + 4.0;
    let track_bottom = track_top + TRACK_HEIGHT;

    // Handle drag
    if response.dragged() {
        let delta = -response.drag_delta().y;
        let range = max - min;
        let speed = range / TRACK_HEIGHT;
        *value = (*value + delta * speed).clamp(min, max);
    }

    // Handle click
    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = 1.0 - ((pos.y - track_top) / TRACK_HEIGHT).clamp(0.0, 1.0);
        *value = min + t * (max - min);
    }

    let range = max - min;
    let t = if range.abs() < f32::EPSILON { 0.0 } else { (*value - min) / range };

    // Track background
    let track_rect = Rect::from_min_size(
        pos2(track_x - TRACK_WIDTH / 2.0, track_top),
        vec2(TRACK_WIDTH, TRACK_HEIGHT),
    );
    painter.rect_filled(track_rect, 3.0, SynthTheme::KNOB_BG);

    // Track fill (from bottom up)
    let fill_height = TRACK_HEIGHT * t;
    if fill_height > 0.5 {
        let fill_rect = Rect::from_min_size(
            pos2(track_x - TRACK_WIDTH / 2.0, track_bottom - fill_height),
            vec2(TRACK_WIDTH, fill_height),
        );
        painter.rect_filled(fill_rect, 3.0, SynthTheme::ACCENT_DARK);
    }

    // Thumb
    let thumb_y = track_bottom - t * TRACK_HEIGHT;
    let thumb_rect = Rect::from_center_size(
        pos2(track_x, thumb_y),
        vec2(THUMB_W, THUMB_H),
    );
    painter.rect_filled(thumb_rect, 3.0, SynthTheme::PANEL_LIGHT);
    painter.rect_stroke(thumb_rect, 3.0, Stroke::new(1.0, SynthTheme::ACCENT), egui::StrokeKind::Inside);

    // Label below
    let label_pos = pos2(rect.center().x, track_bottom + 6.0);
    painter.text(
        label_pos,
        egui::Align2::CENTER_TOP,
        label,
        egui::FontId::proportional(10.0),
        SynthTheme::TEXT_DIM,
    );

    // Value below label
    let value_pos = pos2(rect.center().x, track_bottom + 18.0);
    painter.text(
        value_pos,
        egui::Align2::CENTER_TOP,
        format!("{:.2}", *value),
        egui::FontId::proportional(9.0),
        SynthTheme::TEXT,
    );

    response
}
