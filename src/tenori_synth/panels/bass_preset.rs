use eframe::egui;

use crate::tenori_synth::state::{BassPreset, TenoriState};
use crate::tenori_synth::widgets::panel::synth_panel;
use crate::tenori_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Bass", |ui| {
        let options: Vec<(BassPreset, &str)> =
            BassPreset::ALL.iter().map(|p| (*p, p.label())).collect();
        select_buttons(ui, &mut state.bass_preset, &options);
    });
}
