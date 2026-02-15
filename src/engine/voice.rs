use fundsp::shared::Shared;
use serde::{Deserialize, Serialize};

use super::filter::{FilterConfig, LfoConfig};
use super::oscillator::{AdsrParams, Waveform};

/// Convert a MIDI note number to frequency in Hz.
pub fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

/// Per-voice configuration (topology-changing params trigger audio graph rebuild).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub waveform: Waveform,
    pub adsr: AdsrParams,
    pub filter_cfg: FilterConfig,
    pub cutoff: f32,
    pub resonance: f32,
    pub lfo_cfg: LfoConfig,
    pub lfo_rate: f32,
    pub lfo_depth: f32,
    pub level: f32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            adsr: AdsrParams::default(),
            filter_cfg: FilterConfig::default(),
            cutoff: 1000.0,
            resonance: 0.0,
            lfo_cfg: LfoConfig::default(),
            lfo_rate: 1.0,
            lfo_depth: 0.0,
            level: 1.0,
        }
    }
}

impl VoiceConfig {
    /// Returns true if the topology-changing fields differ (requires graph rebuild).
    pub fn topology_differs(&self, other: &Self) -> bool {
        self.waveform != other.waveform
            || self.adsr != other.adsr
            || self.filter_cfg != other.filter_cfg
            || self.lfo_cfg != other.lfo_cfg
    }
}

/// Per-voice `Shared` atomics for runtime (non-topology) parameters.
pub struct VoiceShared {
    pub cutoff: Shared,
    pub resonance: Shared,
    pub lfo_rate: Shared,
    pub lfo_depth: Shared,
    pub level: Shared,
}

impl VoiceShared {
    pub fn new(cfg: &VoiceConfig) -> Self {
        Self {
            cutoff: Shared::new(cfg.cutoff),
            resonance: Shared::new(cfg.resonance),
            lfo_rate: Shared::new(cfg.lfo_rate),
            lfo_depth: Shared::new(cfg.lfo_depth),
            level: Shared::new(cfg.level),
        }
    }

    /// Sync UI values to atomic Shared params (called every frame).
    pub fn sync(&self, cfg: &VoiceConfig) {
        self.cutoff.set_value(cfg.cutoff);
        self.resonance.set_value(cfg.resonance);
        self.lfo_rate.set_value(cfg.lfo_rate);
        self.lfo_depth.set_value(cfg.lfo_depth);
        self.level.set_value(cfg.level);
    }
}

/// A single synthesizer voice with shared audio parameters.
pub struct Voice {
    pub freq: Shared,
    pub gate: Shared,
    pub velocity: Shared,
    pub note: Option<u8>,
    pub releasing: bool,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            freq: Shared::new(440.0),
            gate: Shared::new(0.0),
            velocity: Shared::new(0.0),
            note: None,
            releasing: false,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.note.is_none() && !self.releasing
    }
}

/// Polyphonic voice allocator managing a fixed number of voices.
pub struct VoiceAllocator {
    pub voices: Vec<Voice>,
    round_robin: usize,
}

impl VoiceAllocator {
    pub fn new(num_voices: usize) -> Self {
        let voices = (0..num_voices).map(|_| Voice::new()).collect();
        Self {
            voices,
            round_robin: 0,
        }
    }

    /// Allocate a voice for a note-on event.
    /// Priority: releasing voice (reuse) > idle voice > round-robin (voice stealing).
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        // Check if this note is already playing — retrigger it
        if let Some(v) = self.voices.iter_mut().find(|v| v.note == Some(note)) {
            v.freq.set_value(midi_note_to_freq(note));
            v.gate.set_value(1.0);
            v.velocity.set_value(velocity as f32 / 127.0);
            v.releasing = false;
            return;
        }

        // Prefer an idle voice so consecutive same-note triggers use separate voices
        // (the releasing voice keeps its ADSR tail while the new voice starts a fresh attack)
        let idx = if let Some(i) = self.voices.iter().position(|v| v.is_idle()) {
            i
        // Then a releasing voice
        } else if let Some(i) = self.voices.iter().position(|v| v.releasing) {
            i
        // Round-robin steal
        } else {
            let i = self.round_robin % self.voices.len();
            self.round_robin = (self.round_robin + 1) % self.voices.len();
            i
        };

        let voice = &mut self.voices[idx];
        voice.freq.set_value(midi_note_to_freq(note));
        voice.gate.set_value(1.0);
        voice.velocity.set_value(velocity as f32 / 127.0);
        voice.note = Some(note);
        voice.releasing = false;
    }

    /// Release a voice matching the given note.
    pub fn note_off(&mut self, note: u8) {
        if let Some(v) = self.voices.iter_mut().find(|v| v.note == Some(note)) {
            v.gate.set_value(0.0);
            v.note = None;
            v.releasing = true;
        }
    }

    /// Force a note onto a specific voice index (used by per-voice Test buttons).
    pub fn force_note_on(&mut self, voice_idx: usize, note: u8, velocity: u8) {
        if let Some(voice) = self.voices.get_mut(voice_idx) {
            voice.freq.set_value(midi_note_to_freq(note));
            voice.gate.set_value(1.0);
            voice.velocity.set_value(velocity as f32 / 127.0);
            voice.note = Some(note);
            voice.releasing = false;
        }
    }

    /// Force note-off on a specific voice index.
    pub fn force_note_off(&mut self, voice_idx: usize) {
        if let Some(voice) = self.voices.get_mut(voice_idx) {
            voice.gate.set_value(0.0);
            voice.note = None;
            voice.releasing = true;
        }
    }

    /// Count how many voices are currently active (playing or releasing).
    #[allow(dead_code)]
    pub fn active_count(&self) -> usize {
        self.voices.iter().filter(|v| !v.is_idle()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_note_to_freq_a4() {
        let freq = midi_note_to_freq(69);
        assert!((freq - 440.0).abs() < 0.01, "A4 should be 440 Hz, got {freq}");
    }

    #[test]
    fn midi_note_to_freq_middle_c() {
        let freq = midi_note_to_freq(60);
        assert!(
            (freq - 261.63).abs() < 0.1,
            "C4 should be ~261.63 Hz, got {freq}"
        );
    }

    #[test]
    fn midi_note_to_freq_octave_doubles() {
        let f1 = midi_note_to_freq(60);
        let f2 = midi_note_to_freq(72);
        assert!(
            (f2 / f1 - 2.0).abs() < 0.01,
            "one octave up should double frequency"
        );
    }

    #[test]
    fn voice_starts_idle() {
        let v = Voice::new();
        assert!(v.is_idle());
        assert_eq!(v.note, None);
        assert!(!v.releasing);
    }

    #[test]
    fn allocator_note_on_assigns_idle_voice() {
        let mut alloc = VoiceAllocator::new(8);
        alloc.note_on(60, 100);
        assert_eq!(alloc.active_count(), 1);
        assert_eq!(alloc.voices[0].note, Some(60));
    }

    #[test]
    fn allocator_note_off_releases_voice() {
        let mut alloc = VoiceAllocator::new(8);
        alloc.note_on(60, 100);
        alloc.note_off(60);
        // Voice is releasing but not idle until release envelope finishes
        assert!(alloc.voices[0].releasing);
        assert_eq!(alloc.voices[0].note, None);
        assert_eq!(alloc.active_count(), 1); // still "active" (releasing)
    }

    #[test]
    fn allocator_prefers_idle_over_releasing() {
        let mut alloc = VoiceAllocator::new(2);
        alloc.note_on(60, 100);
        alloc.note_off(60); // voice 0 is now releasing

        alloc.note_on(64, 100);
        // Should pick voice 1 (idle) over voice 0 (releasing)
        // This ensures consecutive same-note triggers use separate voices
        // so the audio thread sees a proper gate rising edge.
        assert_eq!(alloc.voices[1].note, Some(64));
    }

    #[test]
    fn allocator_steals_releasing_when_no_idle() {
        let mut alloc = VoiceAllocator::new(2);
        alloc.note_on(60, 100);
        alloc.note_on(64, 100);
        // Both voices active
        alloc.note_off(60); // voice 0 releasing
        alloc.note_off(64); // voice 1 releasing

        alloc.note_on(67, 100);
        // Should steal a releasing voice
        let found = alloc.voices.iter().any(|v| v.note == Some(67));
        assert!(found, "should have allocated note 67");
    }

    #[test]
    fn allocator_round_robin_steals_when_all_active() {
        let mut alloc = VoiceAllocator::new(2);
        alloc.note_on(60, 100);
        alloc.note_on(64, 100);
        // Both voices active, no idle, no releasing
        alloc.note_on(67, 100); // must steal
        let found = alloc.voices.iter().any(|v| v.note == Some(67));
        assert!(found, "should have stolen a voice for note 67");
    }

    #[test]
    fn allocator_retrigger_same_note() {
        let mut alloc = VoiceAllocator::new(8);
        alloc.note_on(60, 80);
        alloc.note_on(60, 120);
        // Should not allocate a second voice
        let count = alloc.voices.iter().filter(|v| v.note == Some(60)).count();
        assert_eq!(count, 1, "retriggering same note should reuse the voice");
    }

    #[test]
    fn allocator_polyphony() {
        let mut alloc = VoiceAllocator::new(8);
        for note in 60..68 {
            alloc.note_on(note, 100);
        }
        assert_eq!(alloc.active_count(), 8);
    }

    #[test]
    fn force_note_on_targets_specific_voice() {
        let mut alloc = VoiceAllocator::new(8);
        alloc.force_note_on(3, 60, 100);
        assert_eq!(alloc.voices[3].note, Some(60));
        assert!(!alloc.voices[3].releasing);
        // Other voices should be idle
        assert!(alloc.voices[0].is_idle());
        assert!(alloc.voices[7].is_idle());
    }

    #[test]
    fn force_note_off_releases_specific_voice() {
        let mut alloc = VoiceAllocator::new(8);
        alloc.force_note_on(2, 60, 100);
        alloc.force_note_off(2);
        assert_eq!(alloc.voices[2].note, None);
        assert!(alloc.voices[2].releasing);
    }

    #[test]
    fn voice_config_default() {
        let cfg = VoiceConfig::default();
        assert_eq!(cfg.waveform, Waveform::Sine);
        assert_eq!(cfg.level, 1.0);
        assert_eq!(cfg.cutoff, 1000.0);
        assert_eq!(cfg.resonance, 0.0);
    }

    #[test]
    fn voice_config_topology_differs() {
        let a = VoiceConfig::default();
        let mut b = a.clone();
        assert!(!a.topology_differs(&b));

        b.waveform = Waveform::Saw;
        assert!(a.topology_differs(&b));
    }

    #[test]
    fn voice_config_runtime_change_no_topology_diff() {
        let a = VoiceConfig::default();
        let mut b = a.clone();
        b.cutoff = 5000.0;
        b.resonance = 0.8;
        b.level = 0.5;
        // Runtime-only changes should NOT trigger topology diff
        assert!(!a.topology_differs(&b));
    }
}
