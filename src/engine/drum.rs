use fundsp::prelude32::*;

use super::effects::{wire_chorus, wire_delay, wire_reverb, EffectsConfig, EffectSlot};
use super::filter::{Add2, Mul2, resonance_to_q};
use super::voice::Voice;

// ── Drum sound parameters ──

/// Runtime parameters defining a single percussion sound.
/// All values are set via Shared atomics — no graph rebuild needed.
#[derive(Debug, Clone, Copy)]
pub struct DrumParams {
    pub freq: f32,
    pub noise_level: f32,
    pub sine_level: f32,
    pub cutoff: f32,
    pub resonance: f32,
}

/// 16-instrument drum kit (top row = index 0, bottom row = index 15).
#[allow(dead_code)]
pub const DRUM_KIT: [DrumParams; 16] = [
    DrumParams { freq: 300.0,  noise_level: 0.9,  sine_level: 0.1,  cutoff: 15000.0, resonance: 0.05 }, // Crash
    DrumParams { freq: 500.0,  noise_level: 0.8,  sine_level: 0.2,  cutoff: 10000.0, resonance: 0.2  }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 12000.0, resonance: 0.1  }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 10000.0, resonance: 0.1  }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 3000.0,  resonance: 0.3  }, // Clap
    DrumParams { freq: 800.0,  noise_level: 0.3,  sine_level: 0.7,  cutoff: 6000.0,  resonance: 0.5  }, // Rimshot
    DrumParams { freq: 200.0,  noise_level: 0.7,  sine_level: 0.3,  cutoff: 5000.0,  resonance: 0.2  }, // Snare
    DrumParams { freq: 300.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 1000.0,  resonance: 0.5  }, // Tom Hi
    DrumParams { freq: 200.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 800.0,   resonance: 0.5  }, // Tom Mid
    DrumParams { freq: 120.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 500.0,   resonance: 0.5  }, // Tom Low
    DrumParams { freq: 350.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 1500.0,  resonance: 0.6  }, // Conga Hi
    DrumParams { freq: 200.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 800.0,   resonance: 0.6  }, // Conga Lo
    DrumParams { freq: 560.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 2000.0,  resonance: 0.8  }, // Cowbell
    DrumParams { freq: 2500.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 3000.0,  resonance: 0.9  }, // Claves
    DrumParams { freq: 55.0,   noise_level: 0.1,  sine_level: 0.9,  cutoff: 200.0,   resonance: 0.8  }, // Kick
    DrumParams { freq: 50.0,   noise_level: 0.15, sine_level: 0.95, cutoff: 150.0,   resonance: 0.9  }, // Kick Hard
];

/// Rock kit: punchy kick, snappy snare, bright cymbals, tight hats.
#[allow(dead_code)]
pub const DRUM_KIT_ROCK: [DrumParams; 16] = [
    DrumParams { freq: 280.0,  noise_level: 0.95, sine_level: 0.05, cutoff: 16000.0, resonance: 0.05 }, // Crash
    DrumParams { freq: 520.0,  noise_level: 0.85, sine_level: 0.15, cutoff: 11000.0, resonance: 0.15 }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 13000.0, resonance: 0.08 }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 9000.0,  resonance: 0.12 }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 3500.0,  resonance: 0.25 }, // Clap
    DrumParams { freq: 900.0,  noise_level: 0.25, sine_level: 0.75, cutoff: 7000.0,  resonance: 0.4  }, // Rimshot
    DrumParams { freq: 180.0,  noise_level: 0.65, sine_level: 0.35, cutoff: 6500.0,  resonance: 0.25 }, // Snare
    DrumParams { freq: 320.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 1200.0,  resonance: 0.55 }, // Tom Hi
    DrumParams { freq: 220.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 900.0,   resonance: 0.55 }, // Tom Mid
    DrumParams { freq: 100.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 450.0,   resonance: 0.55 }, // Tom Low
    DrumParams { freq: 350.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 1500.0,  resonance: 0.6  }, // Conga Hi
    DrumParams { freq: 200.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 800.0,   resonance: 0.6  }, // Conga Lo
    DrumParams { freq: 560.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 2000.0,  resonance: 0.8  }, // Cowbell
    DrumParams { freq: 2500.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 3000.0,  resonance: 0.9  }, // Claves
    DrumParams { freq: 45.0,   noise_level: 0.08, sine_level: 0.92, cutoff: 180.0,   resonance: 0.85 }, // Kick
    DrumParams { freq: 42.0,   noise_level: 0.12, sine_level: 0.95, cutoff: 140.0,   resonance: 0.9  }, // Kick Hard
];

/// Jazz kit: soft brush-like textures, warm tones, prominent ride.
#[allow(dead_code)]
pub const DRUM_KIT_JAZZ: [DrumParams; 16] = [
    DrumParams { freq: 320.0,  noise_level: 0.85, sine_level: 0.15, cutoff: 10000.0, resonance: 0.05 }, // Crash
    DrumParams { freq: 480.0,  noise_level: 0.7,  sine_level: 0.3,  cutoff: 12000.0, resonance: 0.25 }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 8000.0,  resonance: 0.08 }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 6000.0,  resonance: 0.1  }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 0.9,  sine_level: 0.1,  cutoff: 2000.0,  resonance: 0.2  }, // Clap
    DrumParams { freq: 700.0,  noise_level: 0.4,  sine_level: 0.6,  cutoff: 4000.0,  resonance: 0.3  }, // Rimshot
    DrumParams { freq: 190.0,  noise_level: 0.8,  sine_level: 0.2,  cutoff: 3000.0,  resonance: 0.15 }, // Snare (brush)
    DrumParams { freq: 280.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 800.0,   resonance: 0.4  }, // Tom Hi
    DrumParams { freq: 180.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 600.0,   resonance: 0.4  }, // Tom Mid
    DrumParams { freq: 110.0,  noise_level: 0.2,  sine_level: 0.8,  cutoff: 400.0,   resonance: 0.4  }, // Tom Low
    DrumParams { freq: 330.0,  noise_level: 0.25, sine_level: 0.75, cutoff: 1200.0,  resonance: 0.5  }, // Conga Hi
    DrumParams { freq: 190.0,  noise_level: 0.25, sine_level: 0.75, cutoff: 700.0,   resonance: 0.5  }, // Conga Lo
    DrumParams { freq: 560.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 2000.0,  resonance: 0.8  }, // Cowbell
    DrumParams { freq: 2500.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 3000.0,  resonance: 0.9  }, // Claves
    DrumParams { freq: 65.0,   noise_level: 0.05, sine_level: 0.95, cutoff: 150.0,   resonance: 0.7  }, // Kick
    DrumParams { freq: 60.0,   noise_level: 0.08, sine_level: 0.92, cutoff: 120.0,   resonance: 0.75 }, // Kick Hard
];

/// Dance/808 kit: deep sub kick, sharp electronic hats, clap-heavy.
#[allow(dead_code)]
pub const DRUM_KIT_DANCE: [DrumParams; 16] = [
    DrumParams { freq: 250.0,  noise_level: 0.9,  sine_level: 0.1,  cutoff: 14000.0, resonance: 0.05 }, // Crash
    DrumParams { freq: 550.0,  noise_level: 0.8,  sine_level: 0.2,  cutoff: 10000.0, resonance: 0.15 }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 15000.0, resonance: 0.05 }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 12000.0, resonance: 0.08 }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 2500.0,  resonance: 0.35 }, // Clap
    DrumParams { freq: 850.0,  noise_level: 0.3,  sine_level: 0.7,  cutoff: 5000.0,  resonance: 0.5  }, // Rimshot
    DrumParams { freq: 160.0,  noise_level: 0.75, sine_level: 0.25, cutoff: 4000.0,  resonance: 0.2  }, // Snare
    DrumParams { freq: 350.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 1000.0,  resonance: 0.6  }, // Tom Hi
    DrumParams { freq: 240.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 700.0,   resonance: 0.6  }, // Tom Mid
    DrumParams { freq: 130.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 400.0,   resonance: 0.6  }, // Tom Low
    DrumParams { freq: 380.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 1800.0,  resonance: 0.55 }, // Conga Hi
    DrumParams { freq: 220.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 900.0,   resonance: 0.55 }, // Conga Lo
    DrumParams { freq: 560.0,  noise_level: 0.05, sine_level: 0.95, cutoff: 2500.0,  resonance: 0.8  }, // Cowbell
    DrumParams { freq: 2500.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 3500.0,  resonance: 0.9  }, // Claves
    DrumParams { freq: 40.0,   noise_level: 0.05, sine_level: 0.98, cutoff: 120.0,   resonance: 0.9  }, // Kick (808)
    DrumParams { freq: 38.0,   noise_level: 0.08, sine_level: 0.98, cutoff: 100.0,   resonance: 0.95 }, // Kick Hard
];

/// Electronic/909 kit: punchy, synthetic, bright hats, tight snare.
#[allow(dead_code)]
pub const DRUM_KIT_ELECTRONIC: [DrumParams; 16] = [
    DrumParams { freq: 350.0,  noise_level: 0.9,  sine_level: 0.1,  cutoff: 18000.0, resonance: 0.03 }, // Crash
    DrumParams { freq: 600.0,  noise_level: 0.75, sine_level: 0.25, cutoff: 14000.0, resonance: 0.1  }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 18000.0, resonance: 0.05 }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 14000.0, resonance: 0.08 }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 4000.0,  resonance: 0.3  }, // Clap
    DrumParams { freq: 1000.0, noise_level: 0.2,  sine_level: 0.8,  cutoff: 8000.0,  resonance: 0.6  }, // Rimshot
    DrumParams { freq: 220.0,  noise_level: 0.85, sine_level: 0.15, cutoff: 7000.0,  resonance: 0.2  }, // Snare
    DrumParams { freq: 340.0,  noise_level: 0.08, sine_level: 0.92, cutoff: 1500.0,  resonance: 0.5  }, // Tom Hi
    DrumParams { freq: 230.0,  noise_level: 0.08, sine_level: 0.92, cutoff: 1000.0,  resonance: 0.5  }, // Tom Mid
    DrumParams { freq: 130.0,  noise_level: 0.08, sine_level: 0.92, cutoff: 600.0,   resonance: 0.5  }, // Tom Low
    DrumParams { freq: 400.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 2000.0,  resonance: 0.65 }, // Conga Hi
    DrumParams { freq: 240.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 1000.0,  resonance: 0.65 }, // Conga Lo
    DrumParams { freq: 580.0,  noise_level: 0.02, sine_level: 0.98, cutoff: 2500.0,  resonance: 0.85 }, // Cowbell
    DrumParams { freq: 2800.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 4000.0,  resonance: 0.9  }, // Claves
    DrumParams { freq: 50.0,   noise_level: 0.12, sine_level: 0.9,  cutoff: 200.0,   resonance: 0.85 }, // Kick
    DrumParams { freq: 48.0,   noise_level: 0.15, sine_level: 0.92, cutoff: 160.0,   resonance: 0.9  }, // Kick Hard
];

/// Latin kit: prominent congas, timbales, warm tones, cowbell/claves forward.
#[allow(dead_code)]
pub const DRUM_KIT_LATIN: [DrumParams; 16] = [
    DrumParams { freq: 300.0,  noise_level: 0.85, sine_level: 0.15, cutoff: 12000.0, resonance: 0.05 }, // Crash
    DrumParams { freq: 500.0,  noise_level: 0.8,  sine_level: 0.2,  cutoff: 10000.0, resonance: 0.2  }, // Ride
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 11000.0, resonance: 0.1  }, // Open HH
    DrumParams { freq: 0.0,    noise_level: 1.0,  sine_level: 0.0,  cutoff: 9000.0,  resonance: 0.1  }, // Closed HH
    DrumParams { freq: 0.0,    noise_level: 0.95, sine_level: 0.05, cutoff: 2800.0,  resonance: 0.3  }, // Clap
    DrumParams { freq: 750.0,  noise_level: 0.35, sine_level: 0.65, cutoff: 5500.0,  resonance: 0.45 }, // Rimshot
    DrumParams { freq: 200.0,  noise_level: 0.6,  sine_level: 0.4,  cutoff: 4500.0,  resonance: 0.2  }, // Snare
    DrumParams { freq: 400.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 2000.0,  resonance: 0.6  }, // Timbale Hi
    DrumParams { freq: 280.0,  noise_level: 0.1,  sine_level: 0.9,  cutoff: 1500.0,  resonance: 0.6  }, // Timbale Lo
    DrumParams { freq: 120.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 500.0,   resonance: 0.5  }, // Tom Low
    DrumParams { freq: 380.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 2000.0,  resonance: 0.7  }, // Conga Hi
    DrumParams { freq: 220.0,  noise_level: 0.15, sine_level: 0.85, cutoff: 1200.0,  resonance: 0.7  }, // Conga Lo
    DrumParams { freq: 580.0,  noise_level: 0.02, sine_level: 0.98, cutoff: 2500.0,  resonance: 0.85 }, // Cowbell
    DrumParams { freq: 2600.0, noise_level: 0.0,  sine_level: 1.0,  cutoff: 3500.0,  resonance: 0.92 }, // Claves
    DrumParams { freq: 55.0,   noise_level: 0.1,  sine_level: 0.9,  cutoff: 200.0,   resonance: 0.8  }, // Kick
    DrumParams { freq: 50.0,   noise_level: 0.15, sine_level: 0.95, cutoff: 150.0,   resonance: 0.9  }, // Kick Hard
];


#[allow(dead_code)]
pub const DRUM_LABELS: [&str; 16] = [
    "Crash", "Ride", "O-HH", "C-HH", "Clap", "Rim", "Snare", "TomH",
    "TomM", "TomL", "CngH", "CngL", "Cowbl", "Clave", "Kick", "KickH",
];

// ── Per-voice shared parameters ──

/// Atomic shared parameters for a single drum voice.
/// Updated per-hit to change the drum sound without graph rebuild.
pub struct DrumVoiceShared {
    pub freq: Shared,
    pub noise_level: Shared,
    pub sine_level: Shared,
    pub cutoff: Shared,
    pub resonance: Shared,
    pub level: Shared,
}

impl DrumVoiceShared {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let default = &DRUM_KIT[14]; // Kick
        Self {
            freq: Shared::new(default.freq),
            noise_level: Shared::new(default.noise_level),
            sine_level: Shared::new(default.sine_level),
            cutoff: Shared::new(default.cutoff),
            resonance: Shared::new(default.resonance),
            level: Shared::new(1.0),
        }
    }

    /// Apply a drum sound's parameters to the shared atomics.
    #[allow(dead_code)]
    pub fn apply_drum(&self, params: &DrumParams) {
        self.freq.set_value(params.freq);
        self.noise_level.set_value(params.noise_level);
        self.sine_level.set_value(params.sine_level);
        self.cutoff.set_value(params.cutoff);
        self.resonance.set_value(params.resonance);
    }
}

// ── Voice graph construction ──

/// Build a single drum voice with fixed topology (no rebuild per hit).
///
/// Signal chain:
///   noise() × noise_level + sine(freq) × sine_level
///   → lowpass(cutoff, Q)
///   → × ADSR(0.001, 0.5, 0.0, 0.1)
///   → × velocity → × level → × master_amp → stereo split
pub fn build_drum_voice_unit(
    gate: &Shared,
    velocity: &Shared,
    master_amp: &Shared,
    shared: &DrumVoiceShared,
) -> Box<dyn AudioUnit> {
    let mut net = Net::new(0, 2);

    // Noise source: noise() × noise_level
    let noise_id = net.push(Box::new(noise()));
    let noise_lvl_id = net.push(Box::new(var(&shared.noise_level)));
    let noise_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(noise_id, 0, noise_mul_id, 0);
    net.connect(noise_lvl_id, 0, noise_mul_id, 1);

    // Sine source: var(freq) >> sine() × sine_level
    let freq_id = net.push(Box::new(var(&shared.freq) >> follow(0.01)));
    let sine_id = net.push(Box::new(sine()));
    net.connect(freq_id, 0, sine_id, 0);
    let sine_lvl_id = net.push(Box::new(var(&shared.sine_level)));
    let sine_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(sine_id, 0, sine_mul_id, 0);
    net.connect(sine_lvl_id, 0, sine_mul_id, 1);

    // Mix: noise + sine
    let mix_id = net.push(Box::new(An(Add2::new())));
    net.connect(noise_mul_id, 0, mix_id, 0);
    net.connect(sine_mul_id, 0, mix_id, 1);

    // Always-on lowpass filter
    let cutoff_id = net.push(Box::new(var(&shared.cutoff) >> follow(0.01)));
    let q_id = net.push(Box::new(var_fn(&shared.resonance, resonance_to_q)));
    let filter_id = net.push(Box::new(lowpass()));
    net.connect(mix_id, 0, filter_id, 0);
    net.connect(cutoff_id, 0, filter_id, 1);
    net.connect(q_id, 0, filter_id, 2);

    // ADSR envelope: fixed short percussive envelope
    let env_id = net.push(Box::new(
        var(gate) >> adsr_live(0.001, 0.5, 0.0, 0.1),
    ));

    // filtered × envelope
    let env_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(filter_id, 0, env_mul_id, 0);
    net.connect(env_id, 0, env_mul_id, 1);

    // × velocity
    let vel_id = net.push(Box::new(var(velocity)));
    let vel_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(env_mul_id, 0, vel_mul_id, 0);
    net.connect(vel_id, 0, vel_mul_id, 1);

    // × per-voice level
    let lvl_id = net.push(Box::new(var(&shared.level) >> follow(0.01)));
    let lvl_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(vel_mul_id, 0, lvl_mul_id, 0);
    net.connect(lvl_id, 0, lvl_mul_id, 1);

    // × master amplitude
    let amp_id = net.push(Box::new(var(master_amp) >> follow(0.01)));
    let amp_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(lvl_mul_id, 0, amp_mul_id, 0);
    net.connect(amp_id, 0, amp_mul_id, 1);

    // Split to stereo
    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(amp_mul_id, 0, split_id, 0);
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    Box::new(net)
}

// ── Polyphonic drum graph ──

/// Build a polyphonic drum graph with 8 voices summed together, plus effects chain.
/// Uses fixed-topology drum voices; different drum sounds are set via Shared parameters.
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn build_drum_poly_graph(
    voices: &[Voice],
    drum_shared: &[DrumVoiceShared],
    master_amp: &Shared,
    effects_cfg: &EffectsConfig,
    delay_time: &Shared,
    delay_feedback: &Shared,
    delay_mix: &Shared,
    reverb_mix: &Shared,
    chorus_mix: &Shared,
) -> (Box<dyn AudioUnit>, Snoop, Snoop) {
    let mut net = Net::new(0, 2);

    let (snoop_l, snoop_backend_l) = snoop(32768);
    let (snoop_r, snoop_backend_r) = snoop(32768);

    let snoop_l_id = net.push(Box::new(snoop_backend_l));
    let snoop_r_id = net.push(Box::new(snoop_backend_r));

    net.connect_output(snoop_l_id, 0, 0);
    net.connect_output(snoop_r_id, 0, 1);

    // Build all drum voice units
    let mut voice_ids: Vec<NodeId> = Vec::new();
    for (i, voice) in voices.iter().enumerate() {
        let shared = &drum_shared[i];
        let unit = build_drum_voice_unit(
            &voice.gate,
            &voice.velocity,
            master_amp,
            shared,
        );
        voice_ids.push(net.push(unit));
    }

    // Sum left channels: chain of Add2 nodes
    let mut sum_l_id = {
        let p = net.push(Box::new(pass()));
        net.connect(voice_ids[0], 0, p, 0);
        p
    };
    for &vid in &voice_ids[1..] {
        let add = net.push(Box::new(An(Add2::new())));
        net.connect(sum_l_id, 0, add, 0);
        net.connect(vid, 0, add, 1);
        sum_l_id = add;
    }

    // Sum right channels: chain of Add2 nodes
    let mut sum_r_id = {
        let p = net.push(Box::new(pass()));
        net.connect(voice_ids[0], 1, p, 0);
        p
    };
    for &vid in &voice_ids[1..] {
        let add = net.push(Box::new(An(Add2::new())));
        net.connect(sum_r_id, 0, add, 0);
        net.connect(vid, 1, add, 1);
        sum_r_id = add;
    }

    // Effects chain
    let mut chain_l = sum_l_id;
    let mut chain_r = sum_r_id;

    for &slot in &effects_cfg.order {
        match slot {
            EffectSlot::Delay if effects_cfg.delay_enabled => {
                chain_l = wire_delay(&mut net, chain_l, delay_time, delay_feedback, delay_mix);
                chain_r = wire_delay(&mut net, chain_r, delay_time, delay_feedback, delay_mix);
            }
            EffectSlot::Reverb if effects_cfg.reverb_enabled => {
                let (rl, rr) = wire_reverb(&mut net, chain_l, chain_r, effects_cfg, reverb_mix);
                chain_l = rl;
                chain_r = rr;
            }
            EffectSlot::Chorus if effects_cfg.chorus_enabled => {
                chain_l = wire_chorus(&mut net, chain_l, effects_cfg, chorus_mix);
                chain_r = wire_chorus(&mut net, chain_r, effects_cfg, chorus_mix);
            }
            _ => {}
        }
    }

    // Connect to snoops
    net.connect(chain_l, 0, snoop_l_id, 0);
    net.connect(chain_r, 0, snoop_r_id, 0);

    (Box::new(net), snoop_l, snoop_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 44100.0;

    #[test]
    fn drum_kit_has_16_entries() {
        assert_eq!(DRUM_KIT.len(), 16);
    }

    #[test]
    fn drum_labels_has_16_entries() {
        assert_eq!(DRUM_LABELS.len(), 16);
    }

    #[test]
    fn all_drum_kits_have_16_entries() {
        assert_eq!(DRUM_KIT_ROCK.len(), 16);
        assert_eq!(DRUM_KIT_JAZZ.len(), 16);
        assert_eq!(DRUM_KIT_DANCE.len(), 16);
        assert_eq!(DRUM_KIT_ELECTRONIC.len(), 16);
        assert_eq!(DRUM_KIT_LATIN.len(), 16);
    }


    #[test]
    fn build_drum_voice_unit_returns_stereo() {
        let gate = Shared::new(0.0);
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = DrumVoiceShared::new();
        let unit = build_drum_voice_unit(&gate, &vel, &master, &shared);
        assert_eq!(unit.inputs(), 0);
        assert_eq!(unit.outputs(), 2);
    }

    #[test]
    fn drum_voice_produces_sound_on_gate() {
        let gate = Shared::new(0.0);
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = DrumVoiceShared::new();
        let mut unit = build_drum_voice_unit(&gate, &vel, &master, &shared);
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Warm up
        for _ in 0..100 {
            unit.get_stereo();
        }

        // Trigger
        gate.set_value(1.0);
        let mut has_nonzero = false;
        for _ in 0..4096 {
            let (l, r) = unit.get_stereo();
            if l != 0.0 || r != 0.0 {
                has_nonzero = true;
                break;
            }
        }
        assert!(has_nonzero, "drum voice should produce sound on gate trigger");
    }

    #[test]
    fn drum_voice_silent_when_gate_off() {
        let gate = Shared::new(0.0);
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = DrumVoiceShared::new();
        let mut unit = build_drum_voice_unit(&gate, &vel, &master, &shared);
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Let it run without trigger
        let mut max_val = 0.0f32;
        for _ in 0..4096 {
            let (l, r) = unit.get_stereo();
            max_val = max_val.max(l.abs()).max(r.abs());
        }
        assert!(
            max_val < 0.01,
            "drum voice with gate off should be near-silent, got {max_val}"
        );
    }

    #[test]
    fn drum_voice_responds_to_param_change() {
        let gate = Shared::new(0.0);
        let vel = Shared::new(1.0);
        let master = Shared::new(0.5);
        let shared = DrumVoiceShared::new();

        // Use snare (higher cutoff, more audible quickly)
        shared.apply_drum(&DRUM_KIT[6]); // Snare: noise=0.7, cutoff=5000

        let mut unit = build_drum_voice_unit(&gate, &vel, &master, &shared);
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Warm up filters
        for _ in 0..256 {
            unit.get_stereo();
        }

        // Trigger snare
        gate.set_value(1.0);
        let mut snare_has_sound = false;
        for _ in 0..8192 {
            let (l, _) = unit.get_stereo();
            if l.abs() > 0.001 {
                snare_has_sound = true;
                break;
            }
        }

        // Switch to closed hi-hat without rebuilding
        gate.set_value(0.0);
        for _ in 0..22050 {
            unit.get_stereo();
        }
        shared.apply_drum(&DRUM_KIT[3]); // Closed HH: noise=1.0, cutoff=10000
        gate.set_value(1.0);
        let mut hh_has_sound = false;
        for _ in 0..8192 {
            let (l, _) = unit.get_stereo();
            if l.abs() > 0.001 {
                hh_has_sound = true;
                break;
            }
        }

        assert!(snare_has_sound, "snare should produce sound");
        assert!(hh_has_sound, "hi-hat should produce sound after param switch");
    }

    #[test]
    fn build_drum_poly_graph_returns_stereo() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let drum_shared: Vec<DrumVoiceShared> = (0..8).map(|_| DrumVoiceShared::new()).collect();
        let master = Shared::new(0.5);
        let ecfg = EffectsConfig::default();
        let dt = Shared::new(0.3);
        let fb = Shared::new(0.3);
        let dm = Shared::new(0.0);
        let rm = Shared::new(0.0);
        let cm = Shared::new(0.0);
        let (graph, _, _) =
            build_drum_poly_graph(&voices, &drum_shared, &master, &ecfg, &dt, &fb, &dm, &rm, &cm);
        assert_eq!(graph.inputs(), 0);
        assert_eq!(graph.outputs(), 2);
    }
}
