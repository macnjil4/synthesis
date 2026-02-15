pub mod effects;
pub mod envelope;
pub mod filter;
pub mod keyboard_panel;
pub mod lfo;
pub mod master;
pub mod oscillator;
pub mod pads_panel;
pub mod voice_strip;

use eframe::egui::{self, RichText};

use crate::synth_ui::theme::SynthTheme;

/// Draw a panel with title and themed frame.
pub fn synth_panel(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    SynthTheme::panel_frame().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.vertical(|ui| {
            ui.label(
                RichText::new(title)
                    .color(SynthTheme::ACCENT_LIGHT)
                    .size(11.0)
                    .strong(),
            );
            ui.add_space(4.0);
            add_contents(ui);
        });
    });
}
