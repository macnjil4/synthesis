use eframe::egui::{self, vec2, Response, Sense, Stroke, Ui};

use crate::synth_ui::theme::SynthTheme;

pub fn select_buttons<T: PartialEq + Copy>(
    ui: &mut Ui,
    current: &mut T,
    options: &[(T, &str)],
) -> Response {
    let mut changed = false;
    let resp = ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for &(val, label) in options {
            let selected = *current == val;
            let text_color = if selected { SynthTheme::TEXT } else { SynthTheme::TEXT_DIM };
            let bg = if selected { SynthTheme::ACCENT_DARK } else { SynthTheme::KNOB_BG };
            let border = if selected { SynthTheme::ACCENT } else { SynthTheme::BORDER };

            let galley = ui.painter().layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(11.0),
                text_color,
            );
            let text_size = galley.size();
            let button_size = vec2(text_size.x + 12.0, 20.0);

            let (rect, resp) = ui.allocate_exact_size(button_size, Sense::click());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 4.0, bg);
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);

            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                text_color,
            );

            if resp.clicked() {
                *current = val;
                changed = true;
            }
        }
    });

    let mut r = resp.response;
    if changed {
        r.mark_changed();
    }
    r
}
