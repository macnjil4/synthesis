use eframe::egui;

use crate::tenori_synth::state::TenoriState;
use crate::tenori_synth::widgets::hslider::hslider;
use crate::tenori_synth::widgets::panel::synth_panel;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Effects", |ui| {
        hslider(ui, "Reverb", &mut state.fx_reverb, 0.0, 100.0);
        hslider(ui, "Delay", &mut state.fx_delay, 0.0, 100.0);
        hslider(ui, "Chorus", &mut state.fx_chorus, 0.0, 100.0);
    });
}
