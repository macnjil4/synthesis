use cpal::Device;
use cpal::SupportedStreamConfig;
use eframe::egui;
use fundsp::shared::Shared;
use fundsp::snoop::Snoop;

use crate::engine;
use crate::engine::oscillator::{AdsrParams, Waveform, build_poly_graph};
use crate::engine::voice::VoiceAllocator;
use crate::midi::{MidiHandler, NoteEvent};

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

    // Change detection for rebuild
    active_waveform: Option<Waveform>,
    active_adsr: Option<AdsrParams>,
}

impl SynthApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (device, supported_config) = engine::init_audio_device();
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
            active_waveform: None,
            active_adsr: None,
        }
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
        );

        let stream = engine::start_stream(&self.device, &self.supported_config, graph);
        self.stream = Some(stream);
        self.snoop_left = Some(snoop_l);
        self.snoop_right = Some(snoop_r);
        self.active_waveform = Some(self.waveform);
        self.active_adsr = Some(self.adsr);
    }

    fn needs_rebuild(&self) -> bool {
        self.active_waveform != Some(self.waveform)
            || self.active_adsr.as_ref() != Some(&self.adsr)
    }

    fn dispatch_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::On { note, velocity } => self.allocator.note_on(note, velocity),
            NoteEvent::Off { note } => self.allocator.note_off(note),
        }
    }
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync master volume
        self.master_amp.set_value(self.amplitude);

        // Process MIDI events
        while let Some(event) = self.midi.try_recv() {
            self.dispatch_event(event);
        }

        // Rebuild stream if waveform or ADSR changed
        if self.playing && self.needs_rebuild() {
            self.rebuild_stream();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synthesis");
            ui.add_space(8.0);

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
                                .selectable_label(self.midi.selected_port() == Some(i), name)
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
            ui.label("Keyboard (C3–B4)");

            let keyboard_events = keyboard::draw(ui, 3);
            for event in keyboard_events {
                self.dispatch_event(event);
            }
        });

        if self.playing {
            ctx.request_repaint();
        }
    }
}
