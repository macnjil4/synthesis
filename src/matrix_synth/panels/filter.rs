use eframe::egui;

use crate::matrix_synth::state::{FilterType, MatrixState};
use crate::matrix_synth::widgets::hslider::hslider;
use crate::matrix_synth::widgets::panel::synth_panel;
use crate::matrix_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState) {
    synth_panel(ui, "Filter", |ui| {
        let options: Vec<(FilterType, &str)> =
            FilterType::ALL.iter().map(|f| (*f, f.label())).collect();
        select_buttons(ui, &mut state.filter_type, &options);
        ui.add_space(8.0);
        hslider(ui, "Cutoff", &mut state.filter_cutoff, 0.0, 100.0);
        hslider(ui, "Reso", &mut state.filter_reso, 0.0, 100.0);
    });
}
