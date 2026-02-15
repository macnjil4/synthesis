use eframe::egui;

use crate::matrix_synth::state::{DrawMode, MatrixState};
use crate::matrix_synth::widgets::panel::synth_panel;
use crate::matrix_synth::widgets::select_buttons::select_buttons;

#[allow(dead_code)]
pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "Draw Mode", |ui| {
        let options: Vec<(DrawMode, &str)> =
            DrawMode::ALL.iter().map(|d| (*d, d.label())).collect();
        select_buttons(ui, &mut state.draw_mode, &options);
    });
}
