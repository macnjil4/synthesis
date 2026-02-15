use eframe::egui;

use crate::matrix_synth::state::{LfoDest, MatrixState};
use crate::matrix_synth::widgets::knob::knob;
use crate::matrix_synth::widgets::panel::synth_panel;
use crate::matrix_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "LFO", |ui| {
        let options: Vec<(LfoDest, &str)> =
            LfoDest::ALL.iter().map(|d| (*d, d.label())).collect();
        select_buttons(ui, &mut state.lfo_dest, &options);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            knob(
                ui,
                "Rate",
                &mut state.lfo_rate,
                0.0,
                100.0,
                30.0,
                "Hz",
                36.0,
            );
            knob(
                ui,
                "Depth",
                &mut state.lfo_depth,
                0.0,
                100.0,
                50.0,
                "",
                36.0,
            );
        });
    });
}
