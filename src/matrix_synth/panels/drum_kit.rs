use eframe::egui;

use crate::matrix_synth::state::{DrumPreset, MatrixState};
use crate::matrix_synth::widgets::knob::knob;
use crate::matrix_synth::widgets::panel::synth_panel;
use crate::matrix_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "Drum Kit", |ui| {
        let options: Vec<(DrumPreset, &str)> =
            DrumPreset::ALL.iter().map(|p| (*p, p.label())).collect();
        select_buttons(ui, &mut state.drum_preset, &options);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            knob(
                ui,
                "Tune",
                &mut state.drum_tune,
                0.0,
                100.0,
                50.0,
                "",
                40.0,
            );
            knob(
                ui,
                "Decay",
                &mut state.drum_decay,
                0.0,
                100.0,
                50.0,
                "",
                40.0,
            );
            knob(
                ui,
                "Color",
                &mut state.drum_color,
                0.0,
                100.0,
                50.0,
                "",
                40.0,
            );
        });
    });
}
