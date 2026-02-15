use eframe::egui::{self, pos2, vec2, Rect, Response, Sense, Stroke, Ui};

use crate::synth_ui::theme::SynthTheme;

const TRACK_HEIGHT: f32 = 8.0;
const THUMB_W: f32 = 12.0;
const THUMB_H: f32 = 16.0;
const LABEL_WIDTH: f32 = 50.0;
const VALUE_WIDTH: f32 = 40.0;

pub fn hslider(ui: &mut Ui, label: &str, value: &mut f32, min: f32, max: f32) -> Response {
    let available = ui.available_width();
    let track_width = (available - LABEL_WIDTH - VALUE_WIDTH - 12.0).max(60.0);
    let total_width = LABEL_WIDTH + track_width + VALUE_WIDTH + 12.0;
    let desired_size = vec2(total_width, 22.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

    let painter = ui.painter_at(rect);
    let cy = rect.center().y;

    // Label on the left
    painter.text(
        pos2(rect.left() + 2.0, cy),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        SynthTheme::TEXT_DIM,
    );

    // Track area
    let track_left = rect.left() + LABEL_WIDTH + 4.0;
    let track_right = track_left + track_width;

    // Handle drag
    if response.dragged() {
        let delta = response.drag_delta().x;
        let range = max - min;
        let speed = range / track_width;
        *value = (*value + delta * speed).clamp(min, max);
    }

    // Handle click
    if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - track_left) / track_width).clamp(0.0, 1.0);
        *value = min + t * (max - min);
    }

    let range = max - min;
    let t = if range.abs() < f32::EPSILON { 0.0 } else { (*value - min) / range };

    // Track background
    let track_rect = Rect::from_min_size(
        pos2(track_left, cy - TRACK_HEIGHT / 2.0),
        vec2(track_width, TRACK_HEIGHT),
    );
    painter.rect_filled(track_rect, 3.0, SynthTheme::KNOB_BG);

    // Track fill
    let fill_width = track_width * t;
    if fill_width > 0.5 {
        let fill_rect = Rect::from_min_size(
            pos2(track_left, cy - TRACK_HEIGHT / 2.0),
            vec2(fill_width, TRACK_HEIGHT),
        );
        painter.rect_filled(fill_rect, 3.0, SynthTheme::ACCENT_DARK);
    }

    // Thumb
    let thumb_x = track_left + t * track_width;
    let thumb_rect = Rect::from_center_size(
        pos2(thumb_x, cy),
        vec2(THUMB_W, THUMB_H),
    );
    painter.rect_filled(thumb_rect, 3.0, SynthTheme::PANEL_LIGHT);
    painter.rect_stroke(thumb_rect, 3.0, Stroke::new(1.0, SynthTheme::ACCENT), egui::StrokeKind::Inside);

    // Value on the right
    painter.text(
        pos2(track_right + 8.0, cy),
        egui::Align2::LEFT_CENTER,
        format!("{:.2}", *value),
        egui::FontId::proportional(10.0),
        SynthTheme::TEXT,
    );

    response
}
