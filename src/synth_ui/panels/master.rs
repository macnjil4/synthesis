use eframe::egui;
use fundsp::snoop::Snoop;

use crate::engine::voice::VoiceAllocator;
use crate::gui::oscilloscope;
use crate::synth_ui::theme::SynthTheme;
use crate::synth_ui::widgets::{level_meter, vslider};

use super::synth_panel;

const VU_WINDOW: usize = 128;

pub fn draw(
    ui: &mut egui::Ui,
    amplitude: &mut f32,
    snoop_left: &mut Option<Snoop>,
    snoop_right: &mut Option<Snoop>,
    allocator: &VoiceAllocator,
) {
    synth_panel(ui, "MASTER", |ui| {
        // Volume slider + VU meters
        ui.horizontal(|ui| {
            vslider(ui, "Vol", amplitude, 0.0, 1.0);

            // Compute RMS levels from snoops
            let level_l = compute_rms(snoop_left);
            let level_r = compute_rms(snoop_right);

            level_meter(ui, level_l);
            level_meter(ui, level_r);
        });

        ui.add_space(4.0);

        // Voice indicators
        ui.label(egui::RichText::new("Voices").color(SynthTheme::TEXT_DIM).size(9.0));
        ui.horizontal(|ui| {
            for voice in &allocator.voices {
                let color = if voice.note.is_some() {
                    SynthTheme::VU_GREEN
                } else if voice.releasing {
                    SynthTheme::VU_YELLOW
                } else {
                    SynthTheme::BORDER
                };
                let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 3.0, color);
            }
        });

        ui.add_space(4.0);

        // Mini oscilloscope
        oscilloscope::draw(ui, snoop_left, snoop_right);
    });
}

fn compute_rms(snoop: &mut Option<Snoop>) -> f32 {
    if let Some(s) = snoop.as_mut() {
        s.update();
        let sum: f32 = (0..VU_WINDOW).map(|i| {
            let v = s.at(i);
            v * v
        }).sum();
        (sum / VU_WINDOW as f32).sqrt().min(1.0)
    } else {
        0.0
    }
}
