use eframe::egui;
use eframe::egui::Stroke;

use crate::tenori_synth::theme::Theme;

/// Mutually exclusive toggle button group.
/// T must implement PartialEq + Copy.
pub fn select_buttons<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    current: &mut T,
    options: &[(T, &str)],
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for &(val, label) in options {
            let is_active = *current == val;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .size(9.0)
                    .strong()
                    .color(if is_active {
                        Theme::TEXT_WHITE
                    } else {
                        Theme::TEXT_DIM
                    }),
            )
            .fill(if is_active {
                Theme::ACCENT_DARK
            } else {
                Theme::KNOB_BG
            })
            .stroke(Stroke::new(
                1.0,
                if is_active {
                    Theme::ACCENT
                } else {
                    Theme::BORDER
                },
            ))
            .corner_radius(4.0)
            .min_size(egui::vec2(0.0, 20.0));

            if ui.add(btn).clicked() {
                *current = val;
            }
        }
    });
}
