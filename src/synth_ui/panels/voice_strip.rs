use eframe::egui::{self, vec2, Stroke};

use crate::engine::filter::{FilterType, LfoTarget};
use crate::engine::oscillator::Waveform;
use crate::engine::voice::{Voice, VoiceConfig};
use crate::midi::NoteEvent;
use crate::synth_ui::theme::SynthTheme;
use crate::synth_ui::widgets::{hslider, knob, select_buttons};

/// Minimum frames a test note stays on (~500ms at 60fps).
const TEST_TRIGGER_FRAMES: u32 = 30;
/// Test note: C4 (middle C).
const TEST_NOTE: u8 = 60;

/// Draw a compact per-voice channel strip.
/// Returns note events for the test button.
pub fn draw(
    ui: &mut egui::Ui,
    idx: usize,
    config: &mut VoiceConfig,
    voice: &Voice,
) -> Vec<NoteEvent> {
    let mut events = Vec::new();

    SynthTheme::panel_frame().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.vertical(|ui| {
            // Header: "VOX N" + status LED
            ui.horizontal(|ui| {
                let title = format!("VOX {}", idx + 1);
                ui.label(
                    egui::RichText::new(title)
                        .color(SynthTheme::ACCENT_LIGHT)
                        .size(10.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (led_rect, _) =
                        ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                    let color = if voice.note.is_some() {
                        SynthTheme::VU_GREEN
                    } else if voice.releasing {
                        SynthTheme::VU_YELLOW
                    } else {
                        SynthTheme::BORDER
                    };
                    ui.painter().circle_filled(led_rect.center(), 3.0, color);
                });
            });

            ui.add_space(2.0);

            // Waveform selector (compact)
            select_buttons::select_buttons(
                ui,
                &mut config.waveform,
                &[
                    (Waveform::Sine, "S"),
                    (Waveform::Saw, "Sw"),
                    (Waveform::Square, "Sq"),
                    (Waveform::Triangle, "T"),
                ],
            );

            ui.add_space(4.0);

            // ADSR — 4 mini knobs
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                knob::mini_knob(ui, "A", &mut config.adsr.attack, 0.001, 2.0);
                knob::mini_knob(ui, "D", &mut config.adsr.decay, 0.001, 2.0);
                knob::mini_knob(ui, "S", &mut config.adsr.sustain, 0.0, 1.0);
                knob::mini_knob(ui, "R", &mut config.adsr.release, 0.001, 5.0);
            });

            ui.add_space(2.0);

            // Filter section
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                let (rect, resp) = ui.allocate_exact_size(vec2(16.0, 14.0), egui::Sense::click());
                let painter = ui.painter_at(rect);
                let (bg, text_c) = if config.filter_cfg.enabled {
                    (SynthTheme::ACCENT_DARK, SynthTheme::TEXT)
                } else {
                    (SynthTheme::KNOB_BG, SynthTheme::TEXT_DIM)
                };
                painter.rect_filled(rect, 3.0, bg);
                painter.rect_stroke(rect, 3.0, Stroke::new(1.0, SynthTheme::BORDER), egui::StrokeKind::Inside);
                painter.text(rect.center(), egui::Align2::CENTER_CENTER, "F", egui::FontId::proportional(8.0), text_c);
                if resp.clicked() {
                    config.filter_cfg.enabled = !config.filter_cfg.enabled;
                }

                select_buttons::select_buttons(
                    ui,
                    &mut config.filter_cfg.filter_type,
                    &[
                        (FilterType::Lowpass, "LP"),
                        (FilterType::Highpass, "HP"),
                        (FilterType::Bandpass, "BP"),
                    ],
                );
            });
            hslider::hslider(ui, "Cut", &mut config.cutoff, 20.0, 20000.0);
            hslider::hslider(ui, "Res", &mut config.resonance, 0.0, 1.0);

            ui.add_space(2.0);

            // LFO section
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                let (rect, resp) = ui.allocate_exact_size(vec2(16.0, 14.0), egui::Sense::click());
                let painter = ui.painter_at(rect);
                let (bg, text_c) = if config.lfo_cfg.enabled {
                    (SynthTheme::ACCENT_DARK, SynthTheme::TEXT)
                } else {
                    (SynthTheme::KNOB_BG, SynthTheme::TEXT_DIM)
                };
                painter.rect_filled(rect, 3.0, bg);
                painter.rect_stroke(rect, 3.0, Stroke::new(1.0, SynthTheme::BORDER), egui::StrokeKind::Inside);
                painter.text(rect.center(), egui::Align2::CENTER_CENTER, "L", egui::FontId::proportional(8.0), text_c);
                if resp.clicked() {
                    config.lfo_cfg.enabled = !config.lfo_cfg.enabled;
                }

                select_buttons::select_buttons(
                    ui,
                    &mut config.lfo_cfg.target,
                    &[
                        (LfoTarget::Frequency, "Fr"),
                        (LfoTarget::Cutoff, "Cu"),
                        (LfoTarget::Amplitude, "Am"),
                    ],
                );
            });
            hslider::hslider(ui, "Rate", &mut config.lfo_rate, 0.1, 20.0);
            hslider::hslider(ui, "Dep", &mut config.lfo_depth, 0.0, 1.0);

            ui.add_space(2.0);

            // Level
            hslider::hslider(ui, "Level", &mut config.level, 0.0, 1.0);

            ui.add_space(4.0);

            // Test button with countdown
            let trigger_id = ui.id().with(("voice_test", idx));
            let remaining: u32 = ui.data(|d| d.get_temp(trigger_id)).unwrap_or(0);

            let is_testing = remaining > 0;

            let btn_text = if is_testing { "..." } else { "TEST" };
            let btn_color = if is_testing { SynthTheme::ACCENT_DARK } else { SynthTheme::KNOB_BG };
            let btn = egui::Button::new(
                egui::RichText::new(btn_text)
                    .color(if is_testing { SynthTheme::TEXT } else { SynthTheme::TEXT_DIM })
                    .size(10.0),
            )
            .fill(btn_color)
            .stroke(Stroke::new(1.0, if is_testing { SynthTheme::ACCENT } else { SynthTheme::BORDER }));

            let resp = ui.add_sized(vec2(ui.available_width(), 20.0), btn);

            if resp.clicked() && !is_testing {
                events.push(NoteEvent::TestOn {
                    voice_idx: idx,
                    note: TEST_NOTE,
                    velocity: 100,
                });
                ui.data_mut(|d| d.insert_temp(trigger_id, TEST_TRIGGER_FRAMES));
            } else if remaining > 0 {
                let new_remaining = remaining - 1;
                if new_remaining == 0 {
                    events.push(NoteEvent::TestOff { voice_idx: idx });
                }
                ui.data_mut(|d| d.insert_temp(trigger_id, new_remaining));
            }
        });
    });

    events
}
