#![allow(dead_code, unused_imports)]

use eframe::egui;

use crate::engine::filter::{LfoConfig, LfoTarget, LfoWaveform};
use crate::synth_ui::theme::SynthTheme;
use crate::synth_ui::widgets::{knob, select_buttons};

use super::synth_panel;

pub fn draw(
    ui: &mut egui::Ui,
    lfo_cfg: &mut LfoConfig,
    lfo_rate: &mut f32,
    lfo_depth: &mut f32,
) {
    synth_panel(ui, "LFO", |ui| {
        ui.horizontal(|ui| {
            ui.checkbox(&mut lfo_cfg.enabled, "");
            ui.label(
                egui::RichText::new(if lfo_cfg.enabled { "ON" } else { "OFF" })
                    .color(if lfo_cfg.enabled {
                        SynthTheme::VU_GREEN
                    } else {
                        SynthTheme::TEXT_DIM
                    })
                    .size(10.0),
            );
        });

        ui.add_space(4.0);

        // Waveform
        select_buttons(ui, &mut lfo_cfg.waveform, &[
            (LfoWaveform::Sine, "SIN"),
            (LfoWaveform::Triangle, "TRI"),
            (LfoWaveform::Saw, "SAW"),
        ]);

        ui.add_space(4.0);

        // Target
        select_buttons(ui, &mut lfo_cfg.target, &[
            (LfoTarget::Frequency, "Freq"),
            (LfoTarget::Cutoff, "Cut"),
            (LfoTarget::Amplitude, "Amp"),
        ]);

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            knob(ui, "Rate", lfo_rate, 0.1, 20.0, "Hz");
            knob(ui, "Depth", lfo_depth, 0.0, 1.0, "");
        });
    });
}
