use eframe::egui;

use crate::matrix_synth::state::{MatrixState, Waveform};
use crate::matrix_synth::widgets::knob::knob;
use crate::matrix_synth::widgets::panel::synth_panel;
use crate::matrix_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "Oscillator", |ui| {
        let options: Vec<(Waveform, &str)> =
            Waveform::ALL.iter().map(|w| (*w, w.label())).collect();
        select_buttons(ui, &mut state.osc_waveform, &options);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            knob(
                ui,
                "Pitch",
                &mut state.osc_pitch,
                0.0,
                100.0,
                50.0,
                "",
                40.0,
            );
            knob(
                ui,
                "Detune",
                &mut state.osc_detune,
                0.0,
                100.0,
                0.0,
                "ct",
                40.0,
            );
        });
    });
}
