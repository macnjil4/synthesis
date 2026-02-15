#![allow(dead_code, unused_imports)]

use eframe::egui;

use crate::engine::oscillator::Waveform;
use crate::synth_ui::widgets::{knob, select_buttons};

use super::synth_panel;

pub fn draw(ui: &mut egui::Ui, waveform: &mut Waveform, pitch: &mut f32, detune: &mut f32) {
    synth_panel(ui, "OSCILLATOR", |ui| {
        // Waveform selector
        select_buttons(ui, waveform, &[
            (Waveform::Sine, "SIN"),
            (Waveform::Saw, "SAW"),
            (Waveform::Square, "SQR"),
            (Waveform::Triangle, "TRI"),
        ]);

        ui.add_space(8.0);

        // Pitch and Detune knobs (UI-only, not connected to engine)
        ui.horizontal(|ui| {
            knob(ui, "Pitch", pitch, -24.0, 24.0, "st");
            knob(ui, "Detune", detune, -50.0, 50.0, "ct");
        });
    });
}
