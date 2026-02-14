use cpal::Device;
use cpal::SupportedStreamConfig;
use eframe::egui;
use fundsp::shared::Shared;
use fundsp::snoop::Snoop;

use crate::engine;
use crate::engine::effects::{EffectSlot, EffectsConfig};
use crate::engine::filter::{FilterConfig, FilterType, LfoConfig, LfoTarget, LfoWaveform};
use crate::engine::oscillator::{AdsrParams, Waveform, build_poly_graph};
use crate::engine::voice::VoiceAllocator;
use crate::midi::{MidiHandler, NoteEvent};
use crate::preset::Preset;

use super::{keyboard, oscilloscope};

pub struct SynthApp {
    device: Device,
    supported_config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,

    waveform: Waveform,
    amplitude: f32,
    playing: bool,
    master_amp: Shared,

    allocator: VoiceAllocator,
    adsr: AdsrParams,

    snoop_left: Option<Snoop>,
    snoop_right: Option<Snoop>,

    midi: MidiHandler,

    // Filter
    filter_cfg: FilterConfig,
    cutoff: f32,
    cutoff_shared: Shared,
    resonance: f32,
    resonance_shared: Shared,

    // LFO
    lfo_cfg: LfoConfig,
    lfo_rate: f32,
    lfo_rate_shared: Shared,
    lfo_depth: f32,
    lfo_depth_shared: Shared,

    // Effects
    effects_cfg: EffectsConfig,
    delay_time: f32,
    delay_time_shared: Shared,
    delay_feedback: f32,
    delay_feedback_shared: Shared,
    delay_mix: f32,
    delay_mix_shared: Shared,
    reverb_mix: f32,
    reverb_mix_shared: Shared,
    chorus_mix: f32,
    chorus_mix_shared: Shared,

    // Change detection for rebuild
    active_waveform: Option<Waveform>,
    active_adsr: Option<AdsrParams>,
    active_filter_cfg: Option<FilterConfig>,
    active_lfo_cfg: Option<LfoConfig>,
    active_effects_cfg: Option<EffectsConfig>,

    // Presets
    preset_names: Vec<String>,
    current_preset: Option<String>,
    save_name: String,
}

impl SynthApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (device, supported_config) = engine::init_audio_device();
        let preset_names = Self::collect_preset_names();
        Self {
            device,
            supported_config,
            stream: None,
            waveform: Waveform::Sine,
            amplitude: 0.5,
            playing: false,
            master_amp: Shared::new(0.5),
            allocator: VoiceAllocator::new(8),
            adsr: AdsrParams::default(),
            snoop_left: None,
            snoop_right: None,
            midi: MidiHandler::new(),
            filter_cfg: FilterConfig::default(),
            cutoff: 1000.0,
            cutoff_shared: Shared::new(1000.0),
            resonance: 0.0,
            resonance_shared: Shared::new(0.0),
            lfo_cfg: LfoConfig::default(),
            lfo_rate: 1.0,
            lfo_rate_shared: Shared::new(1.0),
            lfo_depth: 0.0,
            lfo_depth_shared: Shared::new(0.0),
            effects_cfg: EffectsConfig::default(),
            delay_time: 0.3,
            delay_time_shared: Shared::new(0.3),
            delay_feedback: 0.3,
            delay_feedback_shared: Shared::new(0.3),
            delay_mix: 0.0,
            delay_mix_shared: Shared::new(0.0),
            reverb_mix: 0.0,
            reverb_mix_shared: Shared::new(0.0),
            chorus_mix: 0.0,
            chorus_mix_shared: Shared::new(0.0),
            active_waveform: None,
            active_adsr: None,
            active_filter_cfg: None,
            active_lfo_cfg: None,
            active_effects_cfg: None,
            preset_names,
            current_preset: None,
            save_name: String::new(),
        }
    }

    fn collect_preset_names() -> Vec<String> {
        let mut names: Vec<String> = Preset::factory_presets()
            .iter()
            .map(|p| p.name.clone())
            .collect();
        for user_name in Preset::list_user_presets() {
            if !names.contains(&user_name) {
                names.push(user_name);
            }
        }
        names
    }

    fn start(&mut self) {
        self.rebuild_stream();
        self.playing = true;
    }

    fn stop(&mut self) {
        self.stream = None;
        self.snoop_left = None;
        self.snoop_right = None;
        self.playing = false;
        self.active_waveform = None;
        self.active_adsr = None;
        self.active_filter_cfg = None;
        self.active_lfo_cfg = None;
        self.active_effects_cfg = None;
    }

    fn rebuild_stream(&mut self) {
        self.stream = None;
        self.snoop_left = None;
        self.snoop_right = None;

        let (graph, snoop_l, snoop_r) = build_poly_graph(
            self.waveform,
            &self.allocator.voices,
            &self.master_amp,
            &self.adsr,
            &self.filter_cfg,
            &self.cutoff_shared,
            &self.resonance_shared,
            &self.lfo_cfg,
            &self.lfo_rate_shared,
            &self.lfo_depth_shared,
            &self.effects_cfg,
            &self.delay_time_shared,
            &self.delay_feedback_shared,
            &self.delay_mix_shared,
            &self.reverb_mix_shared,
            &self.chorus_mix_shared,
        );

        let stream = engine::start_stream(&self.device, &self.supported_config, graph);
        self.stream = Some(stream);
        self.snoop_left = Some(snoop_l);
        self.snoop_right = Some(snoop_r);
        self.active_waveform = Some(self.waveform);
        self.active_adsr = Some(self.adsr);
        self.active_filter_cfg = Some(self.filter_cfg);
        self.active_lfo_cfg = Some(self.lfo_cfg);
        self.active_effects_cfg = Some(self.effects_cfg.clone());
    }

    fn needs_rebuild(&self) -> bool {
        self.active_waveform != Some(self.waveform)
            || self.active_adsr.as_ref() != Some(&self.adsr)
            || self.active_filter_cfg.as_ref() != Some(&self.filter_cfg)
            || self.active_lfo_cfg.as_ref() != Some(&self.lfo_cfg)
            || self.active_effects_cfg.as_ref() != Some(&self.effects_cfg)
    }

    fn dispatch_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::On { note, velocity } => self.allocator.note_on(note, velocity),
            NoteEvent::Off { note } => self.allocator.note_off(note),
        }
    }

    fn apply_preset(&mut self, preset: &Preset) {
        self.waveform = preset.waveform;
        self.amplitude = preset.amplitude;
        self.master_amp.set_value(preset.amplitude);
        self.adsr = preset.adsr;
        self.filter_cfg = preset.filter_cfg;
        self.cutoff = preset.cutoff;
        self.cutoff_shared.set_value(preset.cutoff);
        self.resonance = preset.resonance;
        self.resonance_shared.set_value(preset.resonance);
        self.lfo_cfg = preset.lfo_cfg;
        self.lfo_rate = preset.lfo_rate;
        self.lfo_rate_shared.set_value(preset.lfo_rate);
        self.lfo_depth = preset.lfo_depth;
        self.lfo_depth_shared.set_value(preset.lfo_depth);
        self.effects_cfg = preset.effects_cfg.clone();
        self.delay_time = preset.delay_time;
        self.delay_time_shared.set_value(preset.delay_time);
        self.delay_feedback = preset.delay_feedback;
        self.delay_feedback_shared.set_value(preset.delay_feedback);
        self.delay_mix = preset.delay_mix;
        self.delay_mix_shared.set_value(preset.delay_mix);
        self.reverb_mix = preset.reverb_mix;
        self.reverb_mix_shared.set_value(preset.reverb_mix);
        self.chorus_mix = preset.chorus_mix;
        self.chorus_mix_shared.set_value(preset.chorus_mix);
    }

    fn current_to_preset(&self, name: &str) -> Preset {
        Preset {
            name: name.to_string(),
            waveform: self.waveform,
            amplitude: self.amplitude,
            adsr: self.adsr,
            filter_cfg: self.filter_cfg,
            cutoff: self.cutoff,
            resonance: self.resonance,
            lfo_cfg: self.lfo_cfg,
            lfo_rate: self.lfo_rate,
            lfo_depth: self.lfo_depth,
            effects_cfg: self.effects_cfg.clone(),
            delay_time: self.delay_time,
            delay_feedback: self.delay_feedback,
            delay_mix: self.delay_mix,
            reverb_mix: self.reverb_mix,
            chorus_mix: self.chorus_mix,
        }
    }
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync continuous parameters
        self.master_amp.set_value(self.amplitude);
        self.cutoff_shared.set_value(self.cutoff);
        self.resonance_shared.set_value(self.resonance);
        self.lfo_rate_shared.set_value(self.lfo_rate);
        self.lfo_depth_shared.set_value(self.lfo_depth);
        self.delay_time_shared.set_value(self.delay_time);
        self.delay_feedback_shared.set_value(self.delay_feedback);
        self.delay_mix_shared.set_value(self.delay_mix);
        self.reverb_mix_shared.set_value(self.reverb_mix);
        self.chorus_mix_shared.set_value(self.chorus_mix);

        // Process MIDI events
        while let Some(event) = self.midi.try_recv() {
            self.dispatch_event(event);
        }

        // Rebuild stream if topology-changing parameters changed
        if self.playing && self.needs_rebuild() {
            self.rebuild_stream();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Synthesis");
                ui.add_space(8.0);

                // Preset section
                ui.horizontal(|ui| {
                    ui.label("Preset:");
                    let current_label = self
                        .current_preset
                        .as_deref()
                        .unwrap_or("(none)");
                    egui::ComboBox::from_id_salt("preset_selector")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            let names = self.preset_names.clone();
                            for name in &names {
                                if ui
                                    .selectable_label(
                                        self.current_preset.as_deref() == Some(name.as_str()),
                                        name,
                                    )
                                    .clicked()
                                {
                                    // Try factory first, then user
                                    let preset = Preset::factory_presets()
                                        .into_iter()
                                        .find(|p| p.name == *name)
                                        .or_else(|| {
                                            let path =
                                                Preset::presets_dir().join(format!("{name}.json"));
                                            Preset::load(&path).ok()
                                        });
                                    if let Some(p) = preset {
                                        self.apply_preset(&p);
                                        self.current_preset = Some(name.clone());
                                    }
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Save as:");
                    ui.text_edit_singleline(&mut self.save_name);
                    if ui.button("Save").clicked() && !self.save_name.is_empty() {
                        let preset = self.current_to_preset(&self.save_name);
                        let path = Preset::presets_dir()
                            .join(format!("{}.json", self.save_name));
                        if preset.save(&path).is_ok() {
                            self.current_preset = Some(self.save_name.clone());
                            self.preset_names = Self::collect_preset_names();
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Waveform selector
                ui.horizontal(|ui| {
                    ui.label("Waveform:");
                    ui.radio_value(&mut self.waveform, Waveform::Sine, "Sine");
                    ui.radio_value(&mut self.waveform, Waveform::Saw, "Saw");
                    ui.radio_value(&mut self.waveform, Waveform::Square, "Square");
                    ui.radio_value(&mut self.waveform, Waveform::Triangle, "Triangle");
                });

                ui.add_space(4.0);

                // Master volume
                ui.add(
                    egui::Slider::new(&mut self.amplitude, 0.0..=1.0).text("Volume"),
                );

                ui.add_space(4.0);

                // ADSR sliders
                ui.add(
                    egui::Slider::new(&mut self.adsr.attack, 0.001..=2.0)
                        .logarithmic(true)
                        .suffix("s")
                        .text("Attack"),
                );
                ui.add(
                    egui::Slider::new(&mut self.adsr.decay, 0.001..=2.0)
                        .logarithmic(true)
                        .suffix("s")
                        .text("Decay"),
                );
                ui.add(
                    egui::Slider::new(&mut self.adsr.sustain, 0.0..=1.0).text("Sustain"),
                );
                ui.add(
                    egui::Slider::new(&mut self.adsr.release, 0.001..=5.0)
                        .logarithmic(true)
                        .suffix("s")
                        .text("Release"),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Filter section
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.filter_cfg.enabled, "Filter");
                    if self.filter_cfg.enabled {
                        ui.radio_value(
                            &mut self.filter_cfg.filter_type,
                            FilterType::Lowpass,
                            "LP",
                        );
                        ui.radio_value(
                            &mut self.filter_cfg.filter_type,
                            FilterType::Highpass,
                            "HP",
                        );
                        ui.radio_value(
                            &mut self.filter_cfg.filter_type,
                            FilterType::Bandpass,
                            "BP",
                        );
                    }
                });

                if self.filter_cfg.enabled {
                    ui.add(
                        egui::Slider::new(&mut self.cutoff, 20.0..=20000.0)
                            .logarithmic(true)
                            .suffix(" Hz")
                            .text("Cutoff"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.resonance, 0.0..=1.0).text("Resonance"),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // LFO section
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.lfo_cfg.enabled, "LFO");
                    if self.lfo_cfg.enabled {
                        ui.radio_value(
                            &mut self.lfo_cfg.waveform,
                            LfoWaveform::Sine,
                            "Sine",
                        );
                        ui.radio_value(
                            &mut self.lfo_cfg.waveform,
                            LfoWaveform::Triangle,
                            "Tri",
                        );
                        ui.radio_value(
                            &mut self.lfo_cfg.waveform,
                            LfoWaveform::Saw,
                            "Saw",
                        );
                    }
                });

                if self.lfo_cfg.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Target:");
                        ui.radio_value(
                            &mut self.lfo_cfg.target,
                            LfoTarget::Frequency,
                            "Freq",
                        );
                        ui.radio_value(
                            &mut self.lfo_cfg.target,
                            LfoTarget::Cutoff,
                            "Cutoff",
                        );
                        ui.radio_value(
                            &mut self.lfo_cfg.target,
                            LfoTarget::Amplitude,
                            "Amp",
                        );
                    });
                    ui.add(
                        egui::Slider::new(&mut self.lfo_rate, 0.1..=20.0)
                            .suffix(" Hz")
                            .text("Rate"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.lfo_depth, 0.0..=1.0).text("Depth"),
                    );
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Effects section
                ui.label("Effects");
                ui.add_space(4.0);

                // Effect order display/edit
                ui.horizontal(|ui| {
                    ui.label("Order:");
                    for i in 0..3 {
                        let slot_name = match self.effects_cfg.order[i] {
                            EffectSlot::Delay => "Delay",
                            EffectSlot::Reverb => "Reverb",
                            EffectSlot::Chorus => "Chorus",
                        };
                        if i > 0 {
                            ui.label("\u{2192}");
                        }
                        let btn = ui.button(slot_name);
                        if btn.clicked() && i < 2 {
                            self.effects_cfg.order.swap(i, i + 1);
                        }
                    }
                    ui.label("(click to swap)");
                });
                ui.add_space(4.0);

                // Delay
                ui.checkbox(&mut self.effects_cfg.delay_enabled, "Delay");
                if self.effects_cfg.delay_enabled {
                    ui.add(
                        egui::Slider::new(&mut self.delay_time, 0.01..=2.0)
                            .suffix("s")
                            .text("Time"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.delay_feedback, 0.0..=0.99)
                            .text("Feedback"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.delay_mix, 0.0..=1.0).text("Delay Mix"),
                    );
                }

                ui.add_space(4.0);

                // Reverb
                ui.checkbox(&mut self.effects_cfg.reverb_enabled, "Reverb");
                if self.effects_cfg.reverb_enabled {
                    ui.add(
                        egui::Slider::new(&mut self.effects_cfg.reverb_room_size, 1.0..=100.0)
                            .text("Room Size"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.effects_cfg.reverb_time, 0.1..=10.0)
                            .suffix("s")
                            .text("Rev Time"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.reverb_mix, 0.0..=1.0).text("Reverb Mix"),
                    );
                }

                ui.add_space(4.0);

                // Chorus
                ui.checkbox(&mut self.effects_cfg.chorus_enabled, "Chorus");
                if self.effects_cfg.chorus_enabled {
                    ui.add(
                        egui::Slider::new(&mut self.effects_cfg.chorus_separation, 0.0..=1.0)
                            .text("Separation"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.effects_cfg.chorus_variation, 0.0..=1.0)
                            .text("Variation"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.effects_cfg.chorus_mod_freq, 0.1..=10.0)
                            .suffix(" Hz")
                            .text("Mod Freq"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.chorus_mix, 0.0..=1.0).text("Chorus Mix"),
                    );
                }

                ui.add_space(4.0);

                // MIDI section
                ui.horizontal(|ui| {
                    ui.label("MIDI:");
                    let port_names = self.midi.port_names().to_vec();
                    let selected = self.midi.selected_port().unwrap_or(0);
                    let label = if port_names.is_empty() {
                        "No MIDI ports".to_string()
                    } else {
                        port_names.get(selected).cloned().unwrap_or_default()
                    };

                    egui::ComboBox::from_id_salt("midi_port")
                        .selected_text(&label)
                        .show_ui(ui, |ui| {
                            for (i, name) in port_names.iter().enumerate() {
                                if ui
                                    .selectable_label(
                                        self.midi.selected_port() == Some(i),
                                        name,
                                    )
                                    .clicked()
                                {
                                    self.midi.connect(i, Some(ctx.clone()));
                                }
                            }
                        });

                    if self.midi.is_connected() {
                        if ui.button("Disconnect").clicked() {
                            self.midi.disconnect();
                        }
                    } else if ui.button("Refresh").clicked() {
                        self.midi.refresh_ports();
                    }
                });

                // Voice activity indicators
                ui.horizontal(|ui| {
                    ui.label("Voices:");
                    for voice in &self.allocator.voices {
                        let color = if voice.note.is_some() {
                            egui::Color32::from_rgb(100, 200, 100)
                        } else if voice.releasing {
                            egui::Color32::from_rgb(200, 200, 100)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        };
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(12.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(rect.center(), 5.0, color);
                    }
                });

                ui.add_space(4.0);

                // Play/Stop button
                let button_text = if self.playing { "Stop" } else { "Play" };
                if ui.button(button_text).clicked() {
                    if self.playing {
                        self.stop();
                    } else {
                        self.start();
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label("Oscilloscope");

                oscilloscope::draw(ui, &mut self.snoop_left, &mut self.snoop_right);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label("Keyboard (C3\u{2013}B4)");

                let keyboard_events = keyboard::draw(ui, 3);
                for event in keyboard_events {
                    self.dispatch_event(event);
                }
            });
        });

        if self.playing {
            ctx.request_repaint();
        }
    }
}
