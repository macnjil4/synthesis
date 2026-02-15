use cpal::Device;
use cpal::SupportedStreamConfig;
use eframe::egui;
use fundsp::shared::Shared;
use fundsp::snoop::Snoop;

use crate::engine;
use crate::engine::effects::EffectsConfig;
use crate::engine::oscillator::build_poly_graph;
use crate::engine::voice::{VoiceAllocator, VoiceConfig, VoiceShared};
use crate::midi::{MidiHandler, NoteEvent};
use crate::preset::Preset;
use crate::synth_ui::{SynthParams, SynthUI};

pub struct SynthApp {
    device: Device,
    supported_config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,

    playing: bool,
    amplitude: f32,
    master_amp: Shared,

    allocator: VoiceAllocator,

    // Per-voice configs (UI values)
    voice_configs: Vec<VoiceConfig>,
    // Per-voice Shared atomics (synced to audio thread)
    voice_shared: Vec<VoiceShared>,

    snoop_left: Option<Snoop>,
    snoop_right: Option<Snoop>,

    midi: MidiHandler,

    // Effects (global, post-mix)
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

    // Change detection for rebuild (topology-changing configs)
    active_voice_configs: Option<Vec<VoiceConfig>>,
    active_effects_cfg: Option<EffectsConfig>,

    // Presets
    preset_names: Vec<String>,
    current_preset: Option<String>,
    save_name: String,

    // New UI
    synth_ui: SynthUI,
    prev_preset: Option<String>,
}

impl SynthApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let preset_names = Self::collect_preset_names();
        let (device, supported_config) = engine::init_audio_device();
        let voice_configs: Vec<VoiceConfig> = (0..8).map(|_| VoiceConfig::default()).collect();
        let voice_shared: Vec<VoiceShared> =
            voice_configs.iter().map(VoiceShared::new).collect();
        Self {
            device,
            supported_config,
            stream: None,
            playing: false,
            amplitude: 0.5,
            master_amp: Shared::new(0.5),
            allocator: VoiceAllocator::new(8),
            voice_configs,
            voice_shared,
            snoop_left: None,
            snoop_right: None,
            midi: MidiHandler::new(),
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
            active_voice_configs: None,
            active_effects_cfg: None,
            preset_names,
            current_preset: None,
            save_name: String::new(),
            synth_ui: SynthUI::new(),
            prev_preset: None,
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

    #[allow(dead_code)]
    fn stop(&mut self) {
        self.stream = None;
        self.snoop_left = None;
        self.snoop_right = None;
        self.playing = false;
        self.active_voice_configs = None;
        self.active_effects_cfg = None;
    }

    fn rebuild_stream(&mut self) {
        self.stream = None;
        self.snoop_left = None;
        self.snoop_right = None;

        // Rebuild VoiceShared from current configs so new graph uses fresh atomics
        self.voice_shared = self.voice_configs.iter().map(VoiceShared::new).collect();

        let (graph, snoop_l, snoop_r) = build_poly_graph(
            &self.allocator.voices,
            &self.voice_configs,
            &self.voice_shared,
            &self.master_amp,
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
        self.active_voice_configs = Some(self.voice_configs.clone());
        self.active_effects_cfg = Some(self.effects_cfg.clone());
    }

    fn needs_rebuild(&self) -> bool {
        // Check if any voice's topology-changing params differ
        if let Some(ref active) = self.active_voice_configs {
            if active.len() != self.voice_configs.len() {
                return true;
            }
            for (a, b) in active.iter().zip(self.voice_configs.iter()) {
                if a.topology_differs(b) {
                    return true;
                }
            }
        } else {
            return true;
        }
        self.active_effects_cfg.as_ref() != Some(&self.effects_cfg)
    }

    fn dispatch_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::On { note, velocity } => self.allocator.note_on(note, velocity),
            NoteEvent::Off { note } => self.allocator.note_off(note),
            NoteEvent::TestOn { voice_idx, note, velocity } => {
                self.allocator.force_note_on(voice_idx, note, velocity);
            }
            NoteEvent::TestOff { voice_idx } => {
                self.allocator.force_note_off(voice_idx);
            }
        }
    }

    fn apply_preset(&mut self, preset: &Preset) {
        // Apply preset settings to ALL 8 voices
        let cfg = VoiceConfig {
            waveform: preset.waveform,
            adsr: preset.adsr,
            filter_cfg: preset.filter_cfg,
            cutoff: preset.cutoff,
            resonance: preset.resonance,
            lfo_cfg: preset.lfo_cfg,
            lfo_rate: preset.lfo_rate,
            lfo_depth: preset.lfo_depth,
            level: 1.0,
        };
        for vc in &mut self.voice_configs {
            *vc = cfg.clone();
        }

        self.amplitude = preset.amplitude;
        self.master_amp.set_value(preset.amplitude);

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
        // Save voice 1's config as the preset (retro-compatible)
        let vc = &self.voice_configs[0];
        Preset {
            name: name.to_string(),
            waveform: vc.waveform,
            amplitude: self.amplitude,
            adsr: vc.adsr,
            filter_cfg: vc.filter_cfg,
            cutoff: vc.cutoff,
            resonance: vc.resonance,
            lfo_cfg: vc.lfo_cfg,
            lfo_rate: vc.lfo_rate,
            lfo_depth: vc.lfo_depth,
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
        // Process MIDI events
        while let Some(event) = self.midi.try_recv() {
            self.dispatch_event(event);
        }

        // Auto-start playing on first frame
        if !self.playing {
            self.start();
        }

        // Build params struct for UI
        let mut params = SynthParams {
            voice_configs: &mut self.voice_configs,
            amplitude: &mut self.amplitude,
            effects_cfg: &mut self.effects_cfg,
            delay_time: &mut self.delay_time,
            delay_feedback: &mut self.delay_feedback,
            delay_mix: &mut self.delay_mix,
            reverb_mix: &mut self.reverb_mix,
            chorus_mix: &mut self.chorus_mix,
            allocator: &self.allocator,
            snoop_left: &mut self.snoop_left,
            snoop_right: &mut self.snoop_right,
            midi: &mut self.midi,
            playing: self.playing,
            preset_names: &self.preset_names,
            current_preset: &mut self.current_preset,
            save_name: &mut self.save_name,
        };

        // Render UI
        let events = self.synth_ui.show(ctx, &mut params);

        // Dispatch note events from UI
        for event in events {
            self.dispatch_event(event);
        }

        // Handle preset selection change
        if self.current_preset != self.prev_preset {
            if let Some(ref name) = self.current_preset {
                if !self.save_name.is_empty() && name == &self.save_name {
                    let preset = self.current_to_preset(name);
                    let path = Preset::presets_dir().join(format!("{name}.json"));
                    if preset.save(&path).is_ok() {
                        self.preset_names = Self::collect_preset_names();
                    }
                    self.save_name.clear();
                } else {
                    let preset = Preset::factory_presets()
                        .into_iter()
                        .find(|p| p.name == *name)
                        .or_else(|| {
                            let path = Preset::presets_dir().join(format!("{name}.json"));
                            Preset::load(&path).ok()
                        });
                    if let Some(p) = preset {
                        self.apply_preset(&p);
                    }
                }
            }
            self.prev_preset = self.current_preset.clone();
        }

        // Sync continuous parameters to Shared atomics
        self.master_amp.set_value(self.amplitude);
        for (i, shared) in self.voice_shared.iter().enumerate() {
            shared.sync(&self.voice_configs[i]);
        }
        self.delay_time_shared.set_value(self.delay_time);
        self.delay_feedback_shared.set_value(self.delay_feedback);
        self.delay_mix_shared.set_value(self.delay_mix);
        self.reverb_mix_shared.set_value(self.reverb_mix);
        self.chorus_mix_shared.set_value(self.chorus_mix);

        // Rebuild stream if topology-changing parameters changed
        if self.playing && self.needs_rebuild() {
            self.rebuild_stream();
        }

        ctx.request_repaint();
    }
}
