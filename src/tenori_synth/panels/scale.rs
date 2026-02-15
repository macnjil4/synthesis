use eframe::egui;

use crate::tenori_synth::state::{Scale, TenoriState};
use crate::tenori_synth::widgets::panel::synth_panel;
use crate::tenori_synth::widgets::select_buttons::select_buttons;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Scale", |ui| {
        let options: Vec<(Scale, &str)> = Scale::ALL.iter().map(|s| (*s, s.label())).collect();
        select_buttons(ui, &mut state.scale, &options);
    });
}
