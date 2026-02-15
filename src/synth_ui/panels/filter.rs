#![allow(dead_code, unused_imports)]

use eframe::egui;

use crate::engine::filter::{FilterConfig, FilterType};
use crate::synth_ui::theme::SynthTheme;
use crate::synth_ui::widgets::{hslider, select_buttons};

use super::synth_panel;

pub fn draw(
    ui: &mut egui::Ui,
    filter_cfg: &mut FilterConfig,
    cutoff: &mut f32,
    resonance: &mut f32,
) {
    synth_panel(ui, "FILTER", |ui| {
        ui.horizontal(|ui| {
            ui.checkbox(&mut filter_cfg.enabled, "");
            ui.label(
                egui::RichText::new(if filter_cfg.enabled { "ON" } else { "OFF" })
                    .color(if filter_cfg.enabled {
                        SynthTheme::VU_GREEN
                    } else {
                        SynthTheme::TEXT_DIM
                    })
                    .size(10.0),
            );
        });

        ui.add_space(4.0);

        select_buttons(ui, &mut filter_cfg.filter_type, &[
            (FilterType::Lowpass, "LP"),
            (FilterType::Highpass, "HP"),
            (FilterType::Bandpass, "BP"),
        ]);

        ui.add_space(8.0);

        // Cutoff slider (logarithmic feel via manual mapping)
        // We store the actual Hz value but display in log scale
        hslider(ui, "Cutoff", cutoff, 20.0, 20000.0);
        ui.add_space(4.0);
        hslider(ui, "Reso", resonance, 0.0, 1.0);
    });
}
