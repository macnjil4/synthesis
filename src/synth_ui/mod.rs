pub mod panels;
pub mod theme;
pub mod widgets;

use std::collections::HashSet;

use eframe::egui::{self, Event, Key};
use fundsp::snoop::Snoop;

use crate::engine::effects::EffectsConfig;
use crate::engine::voice::{VoiceAllocator, VoiceConfig};
use crate::midi::{MidiHandler, NoteEvent};

use theme::SynthTheme;

/// All synth parameters that the UI reads/writes.
pub struct SynthParams<'a> {
    pub voice_configs: &'a mut [VoiceConfig],
    pub amplitude: &'a mut f32,
    pub effects_cfg: &'a mut EffectsConfig,
    pub delay_time: &'a mut f32,
    pub delay_feedback: &'a mut f32,
    pub delay_mix: &'a mut f32,
    pub reverb_mix: &'a mut f32,
    pub chorus_mix: &'a mut f32,
    pub allocator: &'a VoiceAllocator,
    pub snoop_left: &'a mut Option<Snoop>,
    pub snoop_right: &'a mut Option<Snoop>,
    pub midi: &'a mut MidiHandler,
    pub playing: bool,
    pub preset_names: &'a [String],
    pub current_preset: &'a mut Option<String>,
    pub save_name: &'a mut String,
}

pub struct SynthUI {
    pressed_keys: HashSet<u8>,
    theme_applied: bool,
}

impl SynthUI {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            theme_applied: false,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        params: &mut SynthParams,
    ) -> Vec<NoteEvent> {
        if !self.theme_applied {
            SynthTheme::apply(ctx);
            self.theme_applied = true;
        }

        let mut events = Vec::new();

        // Process keyboard shortcuts
        events.extend(self.process_shortcuts(ctx, params));

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SynthTheme::BG))
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    let (led_rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    let led_color = if params.playing {
                        SynthTheme::VU_GREEN
                    } else {
                        SynthTheme::VU_RED
                    };
                    ui.painter()
                        .circle_filled(led_rect.center(), 4.0, led_color);

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("SYNTHWAVE")
                            .color(SynthTheme::ACCENT_LIGHT)
                            .size(18.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("v0.7.0")
                                .color(SynthTheme::TEXT_DIM)
                                .size(11.0),
                        );
                    });
                });

                ui.add_space(4.0);

                // Preset bar
                self.draw_preset_bar(ui, params);

                ui.add_space(4.0);

                // Voice strips + master row
                let strip_height = ui.available_height() - 200.0; // leave room for keyboard+pads+footer
                let num_voices = params.voice_configs.len();
                let spacing = ui.spacing().item_spacing.x;
                let master_width = 160.0;
                let strip_width = (ui.available_width() - master_width - (num_voices as f32) * spacing) / num_voices as f32;

                ui.horizontal(|ui| {
                    ui.set_height(strip_height);

                    // 8 voice channel strips
                    for i in 0..num_voices {
                        let voice = &params.allocator.voices[i];
                        ui.allocate_ui(egui::vec2(strip_width, strip_height), |ui| {
                            // We need to split the borrow: voice_configs[i] + voice
                            let (_, rest) = params.voice_configs.split_at_mut(i);
                            let (cfg, _) = rest.split_first_mut().unwrap();
                            let strip_events = panels::voice_strip::draw(ui, i, cfg, voice);
                            events.extend(strip_events);
                        });
                    }

                    // Master strip
                    ui.allocate_ui(egui::vec2(master_width, strip_height), |ui| {
                        panels::master::draw(
                            ui,
                            params.amplitude,
                            params.snoop_left,
                            params.snoop_right,
                            params.allocator,
                        );
                        ui.add_space(4.0);
                        panels::effects::draw(
                            ui,
                            params.effects_cfg,
                            params.delay_time,
                            params.delay_feedback,
                            params.delay_mix,
                            params.reverb_mix,
                            params.chorus_mix,
                        );
                    });
                });

                ui.add_space(4.0);

                // Bottom row: keyboard + pads
                let bottom_height = ui.available_height() - 20.0;
                ui.horizontal(|ui| {
                    let kb_width = ui.available_width() * 0.70;
                    let pads_width = ui.available_width() - kb_width - ui.spacing().item_spacing.x;

                    ui.allocate_ui(egui::vec2(kb_width, bottom_height), |ui| {
                        let kb_events = panels::keyboard_panel::draw(ui, &self.pressed_keys);
                        events.extend(kb_events);
                    });
                    ui.allocate_ui(egui::vec2(pads_width, bottom_height), |ui| {
                        let pad_events = panels::pads_panel::draw(ui);
                        events.extend(pad_events);
                    });
                });

                // Footer
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Keys: WXCV... (piano)  Space (panic)  Ctrl+1-4 (waveform all)")
                            .color(SynthTheme::TEXT_DIM)
                            .size(9.0),
                    );
                });
            });

        events
    }

    fn draw_preset_bar(&mut self, ui: &mut egui::Ui, params: &mut SynthParams) {
        SynthTheme::panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Preset").color(SynthTheme::TEXT_DIM).size(11.0));

                let current_label = params
                    .current_preset
                    .as_deref()
                    .unwrap_or("(none)");
                egui::ComboBox::from_id_salt("preset_selector")
                    .selected_text(current_label)
                    .show_ui(ui, |ui| {
                        for name in params.preset_names.iter() {
                            if ui
                                .selectable_label(
                                    params.current_preset.as_deref() == Some(name.as_str()),
                                    name,
                                )
                                .clicked()
                            {
                                *params.current_preset = Some(name.clone());
                            }
                        }
                    });

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Save as").color(SynthTheme::TEXT_DIM).size(11.0));
                ui.add(
                    egui::TextEdit::singleline(params.save_name)
                        .desired_width(120.0)
                        .font(egui::FontId::proportional(11.0)),
                );
                if ui.small_button("Save").clicked() && !params.save_name.is_empty() {
                    *params.current_preset = Some(params.save_name.clone());
                }

                // MIDI section on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if params.midi.is_connected() {
                        if ui.small_button("Disconnect").clicked() {
                            params.midi.disconnect();
                        }
                        ui.label(egui::RichText::new("MIDI connected").color(SynthTheme::VU_GREEN).size(10.0));
                    } else {
                        if ui.small_button("Refresh").clicked() {
                            params.midi.refresh_ports();
                        }
                        let port_names = params.midi.port_names().to_vec();
                        let selected = params.midi.selected_port().unwrap_or(0);
                        let label = if port_names.is_empty() {
                            "No MIDI".to_string()
                        } else {
                            port_names.get(selected).cloned().unwrap_or_default()
                        };
                        egui::ComboBox::from_id_salt("midi_port")
                            .selected_text(&label)
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for (i, name) in port_names.iter().enumerate() {
                                    if ui
                                        .selectable_label(
                                            params.midi.selected_port() == Some(i),
                                            name,
                                        )
                                        .clicked()
                                    {
                                        params.midi.connect(i, Some(ui.ctx().clone()));
                                    }
                                }
                            });
                    }
                });
            });
        });
    }

    fn process_shortcuts(
        &mut self,
        ctx: &egui::Context,
        params: &mut SynthParams,
    ) -> Vec<NoteEvent> {
        let mut events = Vec::new();

        let widget_has_focus = ctx.memory(|mem| mem.focused().is_some());

        ctx.input(|input| {
            // Ctrl+Up/Down: volume
            if input.modifiers.ctrl {
                if input.key_pressed(Key::ArrowUp) {
                    *params.amplitude = (*params.amplitude + 0.05).min(1.0);
                }
                if input.key_pressed(Key::ArrowDown) {
                    *params.amplitude = (*params.amplitude - 0.05).max(0.0);
                }
                // Ctrl+1-4: set waveform on ALL voices
                use crate::engine::oscillator::Waveform;
                let waveform_change = if input.key_pressed(Key::Num1) {
                    Some(Waveform::Sine)
                } else if input.key_pressed(Key::Num2) {
                    Some(Waveform::Saw)
                } else if input.key_pressed(Key::Num3) {
                    Some(Waveform::Square)
                } else if input.key_pressed(Key::Num4) {
                    Some(Waveform::Triangle)
                } else {
                    None
                };
                if let Some(wf) = waveform_change {
                    for vc in params.voice_configs.iter_mut() {
                        vc.waveform = wf;
                    }
                }
            }

            // Tab: cycle filter type on ALL voices (only when no text focus)
            if !widget_has_focus && input.key_pressed(Key::Tab) && !input.modifiers.shift && !params.voice_configs.is_empty() {
                use crate::engine::filter::FilterType;
                let new_type = match params.voice_configs[0].filter_cfg.filter_type {
                    FilterType::Lowpass => FilterType::Highpass,
                    FilterType::Highpass => FilterType::Bandpass,
                    FilterType::Bandpass => FilterType::Lowpass,
                };
                for vc in params.voice_configs.iter_mut() {
                    vc.filter_cfg.filter_type = new_type;
                }
            }
            // Shift+Tab: cycle LFO target on ALL voices
            if !widget_has_focus && input.key_pressed(Key::Tab) && input.modifiers.shift && !params.voice_configs.is_empty() {
                use crate::engine::filter::LfoTarget;
                let new_target = match params.voice_configs[0].lfo_cfg.target {
                    LfoTarget::Frequency => LfoTarget::Cutoff,
                    LfoTarget::Cutoff => LfoTarget::Amplitude,
                    LfoTarget::Amplitude => LfoTarget::Frequency,
                };
                for vc in params.voice_configs.iter_mut() {
                    vc.lfo_cfg.target = new_target;
                }
            }

            // Space: panic (all notes off)
            if !widget_has_focus && input.key_pressed(Key::Space) {
                for &note in &self.pressed_keys.clone() {
                    events.push(NoteEvent::Off { note });
                }
                self.pressed_keys.clear();
            }

            if widget_has_focus || input.modifiers.ctrl {
                return;
            }

            // Piano keyboard mapping (AZERTY-ish, using physical key codes)
            let key_map: &[(Key, u8)] = &[
                (Key::W, 48), (Key::S, 49), (Key::X, 50), (Key::D, 51),
                (Key::C, 52), (Key::V, 53), (Key::G, 54), (Key::B, 55),
                (Key::H, 56), (Key::N, 57), (Key::J, 58), (Key::Comma, 59),
                (Key::A, 60), (Key::Num2, 61), (Key::Z, 62), (Key::Num3, 63),
                (Key::E, 64), (Key::R, 65), (Key::Num5, 66), (Key::T, 67),
                (Key::Num6, 68), (Key::Y, 69), (Key::Num7, 70), (Key::U, 71),
            ];

            let pad_key_map: &[(Key, u8)] = &[
                (Key::F1, 36), (Key::F2, 38), (Key::F3, 42),
                (Key::F4, 39), (Key::F5, 41), (Key::F6, 43),
            ];

            for event in &input.events {
                if let Event::Key { key, physical_key, pressed, repeat, .. } = event {
                    if *repeat {
                        continue;
                    }
                    let effective_key = physical_key.unwrap_or(*key);

                    for &(map_key, note) in key_map {
                        if effective_key == map_key {
                            if *pressed && !self.pressed_keys.contains(&note) {
                                self.pressed_keys.insert(note);
                                events.push(NoteEvent::On { note, velocity: 100 });
                            } else if !pressed && self.pressed_keys.contains(&note) {
                                self.pressed_keys.remove(&note);
                                events.push(NoteEvent::Off { note });
                            }
                        }
                    }

                    for &(map_key, note) in pad_key_map {
                        if effective_key == map_key {
                            if *pressed && !self.pressed_keys.contains(&note) {
                                self.pressed_keys.insert(note);
                                events.push(NoteEvent::On { note, velocity: 120 });
                            } else if !pressed && self.pressed_keys.contains(&note) {
                                self.pressed_keys.remove(&note);
                                events.push(NoteEvent::Off { note });
                            }
                        }
                    }
                }
            }
        });

        events
    }
}
