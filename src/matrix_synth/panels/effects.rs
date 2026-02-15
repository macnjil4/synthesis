use eframe::egui;

use crate::matrix_synth::state::MatrixState;
use crate::matrix_synth::widgets::hslider::hslider;
use crate::matrix_synth::widgets::panel::synth_panel;

pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "Effects", |ui| {
        hslider(ui, "Reverb", &mut state.fx_reverb, 0.0, 100.0);
        hslider(ui, "Delay", &mut state.fx_delay, 0.0, 100.0);
        hslider(ui, "Chorus", &mut state.fx_chorus, 0.0, 100.0);
    });
}
