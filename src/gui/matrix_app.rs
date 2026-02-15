use std::sync::Arc;

use eframe::egui;
use fundsp::shared::Shared;

use crate::engine;
use crate::engine::drum_sample::{load_drum_kit, SampleDrumVoiceShared};
use crate::engine::effects::EffectsConfig;
use crate::engine::filter::{FilterConfig, FilterType, LfoConfig, LfoTarget, LfoWaveform};
use crate::engine::oscillator::{AdsrParams, Waveform};
use crate::engine::matrix::build_matrix_graph;
use crate::engine::voice::{VoiceAllocator, VoiceConfig, VoiceShared};
use crate::matrix_synth::state::{self as ts, BassPreset, DrumPreset};
use crate::matrix_synth::MatrixSynth;

use cpal::{Device, SupportedStreamConfig};

const NUM_VOICES: usize = 8;

fn bass_preset_to_config(preset: BassPreset) -> VoiceConfig {
    let (waveform, attack, decay, sustain, release, cutoff, resonance, lfo_enabled, lfo_rate, lfo_depth) =
        match preset {
            BassPreset::SubBass => (
                Waveform::Sine, 0.01, 0.3, 0.8, 0.3,
                200.0, 0.1, false, 1.0, 0.0,
            ),
            BassPreset::AcidBass => (
                Waveform::Saw, 0.005, 0.2, 0.3, 0.1,
                800.0, 0.7, true, 8.0, 0.6,
            ),
            BassPreset::FunkBass => (
                Waveform::Square, 0.005, 0.15, 0.5, 0.1,
                1500.0, 0.3, false, 1.0, 0.0,
            ),
            BassPreset::WarmBass => (
                Waveform::Triangle, 0.02, 0.4, 0.7, 0.4,
                600.0, 0.2, false, 1.0, 0.0,
            ),
            BassPreset::PluckBass => (
                Waveform::Saw, 0.001, 0.1, 0.0, 0.05,
                3000.0, 0.4, false, 1.0, 0.0,
            ),
            BassPreset::GrowlBass => (
                Waveform::Saw, 0.01, 0.3, 0.6, 0.2,
                1200.0, 0.6, true, 3.0, 0.4,
            ),
        };

    VoiceConfig {
        waveform,
        adsr: AdsrParams { attack, decay, sustain, release },
        filter_cfg: FilterConfig {
            filter_type: FilterType::Lowpass,
            enabled: true,
        },
        cutoff,
        resonance,
        lfo_cfg: LfoConfig {
            waveform: LfoWaveform::Sine,
            target: LfoTarget::Cutoff,
            enabled: lfo_enabled,
        },
        lfo_rate,
        lfo_depth,
        level: 1.0,
    }
}

pub struct MatrixApp {
    matrix: MatrixSynth,

    // Audio engine
    device: Device,
    supported_config: SupportedStreamConfig,
    stream: Option<cpal::Stream>,
    playing: bool,

    // Lead audio state (always active)
    allocator: VoiceAllocator,
    voice_configs: Vec<VoiceConfig>,
    voice_shared: Vec<VoiceShared>,
    master_amp: Shared,
    amplitude: f32,

    // Drum audio state (always active, sample-based)
    drum_allocator: VoiceAllocator,
    drum_shared: Vec<SampleDrumVoiceShared>,
    drum_buffers: Arc<Vec<Vec<f32>>>,
    active_drum_preset: Option<DrumPreset>,

    // Bass audio state (always active)
    bass_allocator: VoiceAllocator,
    bass_configs: Vec<VoiceConfig>,
    bass_shared: Vec<VoiceShared>,

    // Effects (shared between lead, drum, and bass)
    effects_cfg: EffectsConfig,
    delay_time_shared: Shared,
    delay_feedback_shared: Shared,
    delay_mix_shared: Shared,
    reverb_mix_shared: Shared,
    chorus_mix_shared: Shared,

    // Rebuild detection
    active_voice_configs: Option<Vec<VoiceConfig>>,
    active_bass_configs: Option<Vec<VoiceConfig>>,
    active_effects_cfg: Option<EffectsConfig>,

    // Track previous step for note triggering
    prev_step: i32,
    prev_active_notes: Vec<u8>,
    prev_active_drums: Vec<u8>,
    prev_active_bass: Vec<u8>,
}

impl MatrixApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (device, supported_config) = engine::init_audio_device();
        let output_sr = supported_config.sample_rate() as f64;
        let voice_configs: Vec<VoiceConfig> =
            (0..NUM_VOICES).map(|_| VoiceConfig::default()).collect();
        let voice_shared: Vec<VoiceShared> = voice_configs.iter().map(VoiceShared::new).collect();
        let bass_configs: Vec<VoiceConfig> =
            (0..NUM_VOICES).map(|_| bass_preset_to_config(BassPreset::SubBass)).collect();
        let bass_shared: Vec<VoiceShared> = bass_configs.iter().map(VoiceShared::new).collect();

        let default_preset = DrumPreset::LinnDrum;
        let drum_buffers = load_drum_kit(default_preset.dir_name(), output_sr);

        Self {
            matrix: MatrixSynth::new(),
            device,
            supported_config,
            stream: None,
            playing: false,
            allocator: VoiceAllocator::new(NUM_VOICES),
            voice_configs,
            voice_shared,
            master_amp: Shared::new(0.5),
            amplitude: 0.5,
            drum_allocator: VoiceAllocator::new(NUM_VOICES),
            drum_shared: (0..NUM_VOICES).map(|_| SampleDrumVoiceShared::new()).collect(),
            drum_buffers,
            active_drum_preset: None,
            bass_allocator: VoiceAllocator::new(NUM_VOICES),
            bass_configs,
            bass_shared,
            effects_cfg: EffectsConfig::default(),
            delay_time_shared: Shared::new(0.3),
            delay_feedback_shared: Shared::new(0.3),
            delay_mix_shared: Shared::new(0.0),
            reverb_mix_shared: Shared::new(0.0),
            chorus_mix_shared: Shared::new(0.0),
            active_voice_configs: None,
            active_bass_configs: None,
            active_effects_cfg: None,
            prev_step: -1,
            prev_active_notes: Vec::new(),
            prev_active_drums: Vec::new(),
            prev_active_bass: Vec::new(),
        }
    }

    fn start(&mut self) {
        self.rebuild_stream();
        self.playing = true;
    }

    /// Build a combined graph with 8 lead + 8 drum + 8 bass voices playing simultaneously.
    fn rebuild_stream(&mut self) {
        self.stream = None;

        self.voice_shared = self.voice_configs.iter().map(VoiceShared::new).collect();
        self.drum_shared = (0..NUM_VOICES).map(|_| SampleDrumVoiceShared::new()).collect();
        self.bass_shared = self.bass_configs.iter().map(VoiceShared::new).collect();

        let (graph, _, _) = build_matrix_graph(
            &self.allocator.voices,
            &self.voice_configs,
            &self.voice_shared,
            &self.drum_allocator.voices,
            &self.drum_shared,
            &self.drum_buffers,
            &self.bass_allocator.voices,
            &self.bass_configs,
            &self.bass_shared,
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

        self.active_voice_configs = Some(self.voice_configs.clone());
        self.active_bass_configs = Some(self.bass_configs.clone());
        self.active_effects_cfg = Some(self.effects_cfg.clone());
        self.active_drum_preset = Some(self.matrix.state().drum_preset);
    }

    fn needs_rebuild(&self) -> bool {
        // Lead voice topology changes always trigger rebuild (graph always active)
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

        // Bass voice topology changes trigger rebuild
        if let Some(ref active) = self.active_bass_configs {
            if active.len() != self.bass_configs.len() {
                return true;
            }
            for (a, b) in active.iter().zip(self.bass_configs.iter()) {
                if a.topology_differs(b) {
                    return true;
                }
            }
        } else {
            return true;
        }

        // Effects changes trigger rebuild
        if self.active_effects_cfg.as_ref() != Some(&self.effects_cfg) {
            return true;
        }

        // Drum preset change triggers rebuild (new sample buffers)
        if self.active_drum_preset != Some(self.matrix.state().drum_preset) {
            return true;
        }

        false
    }

    /// Map MatrixState synth parameters to engine VoiceConfig.
    /// Lead params are always synced (graph always active). Effects sync always runs.
    fn sync_voice_configs_from_matrix(&mut self) {
        let state = self.matrix.state();

        // Always sync lead params (lead graph always active)
        let waveform = match state.osc_waveform {
            ts::Waveform::Sine => Waveform::Sine,
            ts::Waveform::Saw => Waveform::Saw,
            ts::Waveform::Square => Waveform::Square,
            ts::Waveform::Tri => Waveform::Triangle,
        };

        let attack = (state.env_attack / 100.0 * 2.0).max(0.001);
        let decay = (state.env_decay / 100.0 * 2.0).max(0.001);
        let sustain = state.env_sustain / 100.0;
        let release = (state.env_release / 100.0 * 2.0).max(0.001);

        let adsr = AdsrParams {
            attack,
            decay,
            sustain,
            release,
        };

        let filter_type = match state.filter_type {
            ts::FilterType::LP => FilterType::Lowpass,
            ts::FilterType::HP => FilterType::Highpass,
            ts::FilterType::BP => FilterType::Bandpass,
        };

        let cutoff_t = state.filter_cutoff / 100.0;
        let cutoff = 20.0 * (1000.0_f32).powf(cutoff_t);
        let resonance = state.filter_reso / 100.0;

        let filter_cfg = FilterConfig {
            filter_type,
            enabled: true,
        };

        let lfo_target = match state.lfo_dest {
            ts::LfoDest::Pitch => LfoTarget::Frequency,
            ts::LfoDest::Filter => LfoTarget::Cutoff,
            ts::LfoDest::Amp => LfoTarget::Amplitude,
        };

        let lfo_rate = state.lfo_rate / 100.0 * 20.0;
        let lfo_depth = state.lfo_depth / 100.0;

        let lfo_cfg = LfoConfig {
            waveform: LfoWaveform::Sine,
            target: lfo_target,
            enabled: state.lfo_depth > 1.0,
        };

        let cfg = VoiceConfig {
            waveform,
            adsr,
            filter_cfg,
            cutoff,
            resonance,
            lfo_cfg,
            lfo_rate,
            lfo_depth,
            level: 1.0,
        };

        for vc in &mut self.voice_configs {
            *vc = cfg.clone();
        }

        // Bass preset sync (always active)
        let bass_cfg = bass_preset_to_config(state.bass_preset);
        for vc in &mut self.bass_configs {
            *vc = bass_cfg.clone();
        }

        // Reload drum samples if preset changed (before graph rebuild)
        if self.active_drum_preset != Some(state.drum_preset) {
            let output_sr = self.supported_config.sample_rate() as f64;
            self.drum_buffers = load_drum_kit(state.drum_preset.dir_name(), output_sr);
        }

        // Effects sync (always)
        let delay_mix = state.fx_delay / 100.0;
        let reverb_mix = state.fx_reverb / 100.0;
        let chorus_mix = state.fx_chorus / 100.0;

        self.effects_cfg.delay_enabled = delay_mix > 0.01;
        self.effects_cfg.reverb_enabled = reverb_mix > 0.01;
        self.effects_cfg.chorus_enabled = chorus_mix > 0.01;

        self.delay_mix_shared.set_value(delay_mix);
        self.reverb_mix_shared.set_value(reverb_mix);
        self.chorus_mix_shared.set_value(chorus_mix);
    }

    /// Handle playhead step changes: trigger BOTH lead and drum notes simultaneously.
    fn handle_step_change(&mut self) {
        let state = self.matrix.state();
        let current_step = state.play_col;

        if current_step == self.prev_step {
            return;
        }

        // ── Lead notes ──
        for &note in &self.prev_active_notes {
            self.allocator.note_off(note);
        }
        let mut new_lead_active = Vec::new();
        if current_step >= 0 && state.is_playing {
            let col = current_step as usize;
            for row in 0..ts::ROWS {
                if state.grid[row][col] && !state.lead_row_mute[row] {
                    let vel = (100.0 * state.lead_row_volume[row]) as u8;
                    let midi_note = state.row_to_midi(row);
                    self.allocator.note_on(midi_note, vel.max(1));
                    new_lead_active.push(midi_note);
                }
            }
        }
        self.prev_active_notes = new_lead_active;

        // ── Drum notes (sample-based) ──
        for &drum_id in &self.prev_active_drums {
            self.drum_allocator.note_off(drum_id);
        }
        let mut new_drum_active = Vec::new();
        if current_step >= 0 && state.is_playing {
            let col = current_step as usize;
            let tune = (state.drum_tune - 50.0) / 50.0; // -1.0 .. +1.0
            let pitch_ratio = (1.0 + tune * 0.5).clamp(0.5, 2.0);

            for row in 0..ts::ROWS {
                if state.drum_grid[row][col] && !state.drum_row_mute[row] {
                    let vel = (100.0 * state.drum_row_volume[row]) as u8;
                    let drum_id = row as u8;
                    self.drum_allocator.note_on(drum_id, vel.max(1));

                    if let Some(voice_idx) = self
                        .drum_allocator
                        .voices
                        .iter()
                        .position(|v| v.note == Some(drum_id))
                    {
                        let shared = &self.drum_shared[voice_idx];
                        shared.sample_index.set_value(row as f32);
                        shared.pitch_ratio.set_value(pitch_ratio);
                        shared.trigger.set_value(shared.trigger.value() + 1.0);
                    }

                    new_drum_active.push(drum_id);
                }
            }
        }
        self.prev_active_drums = new_drum_active;

        // ── Bass notes ──
        for &note in &self.prev_active_bass {
            self.bass_allocator.note_off(note);
        }
        let mut new_bass_active = Vec::new();
        if current_step >= 0 && state.is_playing {
            let col = current_step as usize;
            for row in 0..ts::ROWS {
                if state.bass_grid[row][col] && !state.bass_row_mute[row] {
                    let vel = (100.0 * state.bass_row_volume[row]) as u8;
                    let midi_note = state.row_to_bass_midi(row);
                    self.bass_allocator.note_on(midi_note, vel.max(1));
                    new_bass_active.push(midi_note);
                }
            }
        }
        self.prev_active_bass = new_bass_active;

        self.prev_step = current_step;
    }
}

impl eframe::App for MatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-start
        if !self.playing {
            self.start();
        }

        // Render UI
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.matrix.show(ctx, ui);
            });

        // Map matrix state to engine params
        self.sync_voice_configs_from_matrix();

        // Handle step changes (trigger both lead and drum notes)
        self.handle_step_change();

        // Sync shared params (always, all graphs active)
        self.master_amp.set_value(self.amplitude);
        for (i, shared) in self.voice_shared.iter().enumerate() {
            shared.sync(&self.voice_configs[i]);
        }
        for (i, shared) in self.bass_shared.iter().enumerate() {
            shared.sync(&self.bass_configs[i]);
        }

        // Rebuild if needed
        if self.playing && self.needs_rebuild() {
            self.rebuild_stream();
        }

        ctx.request_repaint();
    }
}
