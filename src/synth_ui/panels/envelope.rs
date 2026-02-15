#![allow(dead_code, unused_imports)]

use eframe::egui;

use crate::engine::oscillator::AdsrParams;
use crate::synth_ui::widgets::vslider;

use super::synth_panel;

pub fn draw(ui: &mut egui::Ui, adsr: &mut AdsrParams) {
    synth_panel(ui, "ENVELOPE", |ui| {
        ui.horizontal(|ui| {
            vslider(ui, "A", &mut adsr.attack, 0.001, 2.0);
            vslider(ui, "D", &mut adsr.decay, 0.001, 2.0);
            vslider(ui, "S", &mut adsr.sustain, 0.0, 1.0);
            vslider(ui, "R", &mut adsr.release, 0.001, 5.0);
        });
    });
}
