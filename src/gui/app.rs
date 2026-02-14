use cpal::Device;
use cpal::SupportedStreamConfig;
use eframe::egui;
use fundsp::shared::Shared;
use fundsp::snoop::Snoop;

use crate::engine;
use crate::engine::oscillator::{Waveform, build_oscillator_shared};

use super::oscilloscope;

pub struct SynthApp {
    device: Device,
    supported_config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,
    waveform: Waveform,
    frequency: f32,
    amplitude: f32,
    playing: bool,
    freq_shared: Shared,
    amp_shared: Shared,
    snoop_left: Option<Snoop>,
    snoop_right: Option<Snoop>,
    active_waveform: Option<Waveform>,
}

impl SynthApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (device, supported_config) = engine::init_audio_device();
        Self {
            device,
            supported_config,
            stream: None,
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude: 0.5,
            playing: false,
            freq_shared: Shared::new(440.0),
            amp_shared: Shared::new(0.5),
            snoop_left: None,
            snoop_right: None,
            active_waveform: None,
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
    }

    fn rebuild_stream(&mut self) {
        self.stream = None;
        self.snoop_left = None;
        self.snoop_right = None;

        let (graph, snoop_l, snoop_r) =
            build_oscillator_shared(self.waveform, &self.freq_shared, &self.amp_shared);

        let stream = engine::start_stream(&self.device, &self.supported_config, graph);
        self.stream = Some(stream);
        self.snoop_left = Some(snoop_l);
        self.snoop_right = Some(snoop_r);
        self.active_waveform = Some(self.waveform);
    }
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.freq_shared.set_value(self.frequency);
        self.amp_shared.set_value(self.amplitude);

        if self.playing && self.active_waveform != Some(self.waveform) {
            self.rebuild_stream();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synthesis");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Waveform:");
                ui.radio_value(&mut self.waveform, Waveform::Sine, "Sine");
                ui.radio_value(&mut self.waveform, Waveform::Saw, "Saw");
                ui.radio_value(&mut self.waveform, Waveform::Square, "Square");
                ui.radio_value(&mut self.waveform, Waveform::Triangle, "Triangle");
            });

            ui.add_space(4.0);

            ui.add(
                egui::Slider::new(&mut self.frequency, 20.0..=20000.0)
                    .logarithmic(true)
                    .text("Frequency (Hz)"),
            );

            ui.add(
                egui::Slider::new(&mut self.amplitude, 0.0..=1.0)
                    .text("Volume"),
            );

            ui.add_space(8.0);

            let button_text = if self.playing { "Stop" } else { "Play" };
            if ui.button(button_text).clicked() {
                if self.playing {
                    self.stop();
                } else {
                    self.start();
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label("Oscilloscope");

            oscilloscope::draw(ui, &mut self.snoop_left, &mut self.snoop_right);
        });

        if self.playing {
            ctx.request_repaint();
        }
    }
}
