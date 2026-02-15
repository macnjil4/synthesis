use eframe::egui;

use crate::tenori_synth::state::TenoriState;
use crate::tenori_synth::widgets::knob::knob;
use crate::tenori_synth::widgets::panel::synth_panel;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Envelope", |ui| {
        ui.horizontal(|ui| {
            knob(ui, "A", &mut state.env_attack, 0.0, 100.0, 10.0, "", 36.0);
            knob(ui, "D", &mut state.env_decay, 0.0, 100.0, 30.0, "", 36.0);
            knob(
                ui,
                "S",
                &mut state.env_sustain,
                0.0,
                100.0,
                70.0,
                "",
                36.0,
            );
            knob(
                ui,
                "R",
                &mut state.env_release,
                0.0,
                100.0,
                40.0,
                "",
                36.0,
            );
        });
    });
}
