use eframe::egui;

use crate::engine::effects::{EffectSlot, EffectsConfig};
use crate::synth_ui::theme::SynthTheme;
use crate::synth_ui::widgets::hslider;

use super::synth_panel;

pub fn draw(
    ui: &mut egui::Ui,
    effects_cfg: &mut EffectsConfig,
    delay_time: &mut f32,
    delay_feedback: &mut f32,
    delay_mix: &mut f32,
    reverb_mix: &mut f32,
    chorus_mix: &mut f32,
) {
    synth_panel(ui, "EFFECTS", |ui| {
        // Effect order display
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Chain:").color(SynthTheme::TEXT_DIM).size(10.0));
            for i in 0..3 {
                let slot_name = match effects_cfg.order[i] {
                    EffectSlot::Delay => "DLY",
                    EffectSlot::Reverb => "REV",
                    EffectSlot::Chorus => "CHR",
                };
                if i > 0 {
                    ui.label(egui::RichText::new("\u{2192}").color(SynthTheme::TEXT_DIM).size(10.0));
                }
                if ui.small_button(slot_name).clicked() && i < 2 {
                    effects_cfg.order.swap(i, i + 1);
                }
            }
        });

        ui.add_space(4.0);

        // Delay
        ui.horizontal(|ui| {
            ui.checkbox(&mut effects_cfg.delay_enabled, "");
            ui.label(
                egui::RichText::new("DELAY")
                    .color(if effects_cfg.delay_enabled {
                        SynthTheme::ACCENT_LIGHT
                    } else {
                        SynthTheme::TEXT_DIM
                    })
                    .size(10.0)
                    .strong(),
            );
        });
        if effects_cfg.delay_enabled {
            hslider(ui, "Time", delay_time, 0.01, 2.0);
            hslider(ui, "Fback", delay_feedback, 0.0, 0.99);
            hslider(ui, "Mix", delay_mix, 0.0, 1.0);
        }

        ui.add_space(4.0);

        // Reverb
        ui.horizontal(|ui| {
            ui.checkbox(&mut effects_cfg.reverb_enabled, "");
            ui.label(
                egui::RichText::new("REVERB")
                    .color(if effects_cfg.reverb_enabled {
                        SynthTheme::ACCENT_LIGHT
                    } else {
                        SynthTheme::TEXT_DIM
                    })
                    .size(10.0)
                    .strong(),
            );
        });
        if effects_cfg.reverb_enabled {
            hslider(ui, "Room", &mut effects_cfg.reverb_room_size, 1.0, 100.0);
            hslider(ui, "Time", &mut effects_cfg.reverb_time, 0.1, 10.0);
            hslider(ui, "Mix", reverb_mix, 0.0, 1.0);
        }

        ui.add_space(4.0);

        // Chorus
        ui.horizontal(|ui| {
            ui.checkbox(&mut effects_cfg.chorus_enabled, "");
            ui.label(
                egui::RichText::new("CHORUS")
                    .color(if effects_cfg.chorus_enabled {
                        SynthTheme::ACCENT_LIGHT
                    } else {
                        SynthTheme::TEXT_DIM
                    })
                    .size(10.0)
                    .strong(),
            );
        });
        if effects_cfg.chorus_enabled {
            hslider(ui, "Sep", &mut effects_cfg.chorus_separation, 0.0, 1.0);
            hslider(ui, "Var", &mut effects_cfg.chorus_variation, 0.0, 1.0);
            hslider(ui, "ModFq", &mut effects_cfg.chorus_mod_freq, 0.1, 10.0);
            hslider(ui, "Mix", chorus_mix, 0.0, 1.0);
        }
    });
}
