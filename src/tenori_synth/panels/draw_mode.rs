use eframe::egui;

use crate::tenori_synth::state::{DrawMode, TenoriState};
use crate::tenori_synth::widgets::panel::synth_panel;
use crate::tenori_synth::widgets::select_buttons::select_buttons;

#[allow(dead_code)]
pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState) {
    synth_panel(ui, "Draw Mode", |ui| {
        let options: Vec<(DrawMode, &str)> =
            DrawMode::ALL.iter().map(|d| (*d, d.label())).collect();
        select_buttons(ui, &mut state.draw_mode, &options);
    });
}
