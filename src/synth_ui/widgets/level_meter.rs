use eframe::egui::{self, pos2, vec2, Rect, Ui};

use crate::synth_ui::theme::SynthTheme;

const METER_WIDTH: f32 = 12.0;
const METER_HEIGHT: f32 = 90.0;
const NUM_SEGMENTS: usize = 15;

pub fn level_meter(ui: &mut Ui, value: f32) {
    let desired_size = vec2(METER_WIDTH, METER_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 2.0, SynthTheme::KNOB_BG);

    let segment_height = METER_HEIGHT / NUM_SEGMENTS as f32;
    let gap = 1.0;
    let active_segments = ((value.clamp(0.0, 1.0) * NUM_SEGMENTS as f32).ceil() as usize).min(NUM_SEGMENTS);

    for i in 0..NUM_SEGMENTS {
        let seg_bottom = rect.bottom() - i as f32 * segment_height;
        let seg_top = seg_bottom - segment_height + gap;

        let seg_rect = Rect::from_min_max(
            pos2(rect.left() + 1.0, seg_top),
            pos2(rect.right() - 1.0, seg_bottom - gap),
        );

        let frac = i as f32 / NUM_SEGMENTS as f32;
        let color = if i < active_segments {
            if frac > 0.85 {
                SynthTheme::VU_RED
            } else if frac > 0.65 {
                SynthTheme::VU_YELLOW
            } else {
                SynthTheme::VU_GREEN
            }
        } else {
            SynthTheme::BORDER
        };

        painter.rect_filled(seg_rect, 1.0, color);
    }
}
