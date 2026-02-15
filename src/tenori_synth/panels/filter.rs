use eframe::egui;

use crate::tenori_synth::state::{FilterType, TenoriState};
use crate::tenori_synth::widgets::hslider::hslider;
use crate::tenori_synth::widgets::panel::synth_panel;
use crate::tenori_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Filter", |ui| {
        let options: Vec<(FilterType, &str)> =
            FilterType::ALL.iter().map(|f| (*f, f.label())).collect();
        select_buttons(ui, &mut state.filter_type, &options);
        ui.add_space(8.0);
        hslider(ui, "Cutoff", &mut state.filter_cutoff, 0.0, 100.0);
        hslider(ui, "Reso", &mut state.filter_reso, 0.0, 100.0);
    });
}
