use fundsp::prelude32::*;
use serde::{Deserialize, Serialize};

use super::effects::{EffectsConfig, EffectSlot, wire_delay, wire_reverb, wire_chorus};
use super::filter::{
    Add2, FilterConfig, FilterType, LfoConfig, LfoTarget, LfoWaveform, Mul2, resonance_to_q,
};
use super::voice::Voice;

/// ADSR envelope parameters.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdsrParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl std::fmt::Display for Waveform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Waveform::Sine => write!(f, "sine"),
            Waveform::Saw => write!(f, "saw"),
            Waveform::Square => write!(f, "square"),
            Waveform::Triangle => write!(f, "triangle"),
        }
    }
}

/// Build a fundsp audio graph for the given waveform, frequency, and amplitude.
/// Returns a boxed stereo `AudioUnit` (mono oscillator duplicated to both channels).
pub fn build_oscillator(waveform: Waveform, frequency: f32, amplitude: f32) -> Box<dyn AudioUnit> {
    let freq = frequency;
    let amp = amplitude;

    match waveform {
        Waveform::Sine => Box::new(dc(freq) >> (sine() * dc(amp)) >> split::<U2>()),
        Waveform::Saw => Box::new(dc(freq) >> (saw() * dc(amp)) >> split::<U2>()),
        Waveform::Square => Box::new(dc(freq) >> (square() * dc(amp)) >> split::<U2>()),
        Waveform::Triangle => Box::new(dc(freq) >> (triangle() * dc(amp)) >> split::<U2>()),
    }
}

/// Build a fundsp audio graph with shared (atomic) parameters and snoop outputs.
/// Returns the graph plus left/right Snoop frontends for oscilloscope visualization.
#[cfg(test)]
pub fn build_oscillator_shared(
    waveform: Waveform,
    freq_shared: &Shared,
    amp_shared: &Shared,
) -> (Box<dyn AudioUnit>, Snoop, Snoop) {
    let (snoop_left, snoop_backend_left) = snoop(32768);
    let (snoop_right, snoop_backend_right) = snoop(32768);

    let freq_control = var(freq_shared) >> follow(0.01);
    let amp_control = var(amp_shared) >> follow(0.01);

    let graph: Box<dyn AudioUnit> = match waveform {
        Waveform::Sine => Box::new(
            freq_control >> (sine() * amp_control) >> split::<U2>()
                >> (snoop_backend_left | snoop_backend_right),
        ),
        Waveform::Saw => Box::new(
            freq_control >> (saw() * amp_control) >> split::<U2>()
                >> (snoop_backend_left | snoop_backend_right),
        ),
        Waveform::Square => Box::new(
            freq_control >> (square() * amp_control) >> split::<U2>()
                >> (snoop_backend_left | snoop_backend_right),
        ),
        Waveform::Triangle => Box::new(
            freq_control >> (triangle() * amp_control) >> split::<U2>()
                >> (snoop_backend_left | snoop_backend_right),
        ),
    };

    (graph, snoop_left, snoop_right)
}

/// Build an LFO modulation node inside a Net.
/// Returns the NodeId of a 0-input, 1-output node that outputs values around 1.0.
/// Formula: dc(1.0) + (lfo_osc × depth)
fn build_lfo_mod(
    net: &mut Net,
    lfo_cfg: &LfoConfig,
    lfo_rate: &Shared,
    lfo_depth: &Shared,
) -> NodeId {
    // DC offset of 1.0
    let dc_id = net.push(Box::new(dc(1.0)));

    // LFO oscillator: rate >> oscillator
    let rate_id = net.push(Box::new(var(lfo_rate) >> follow(0.01)));
    let osc_id = match lfo_cfg.waveform {
        LfoWaveform::Sine => net.push(Box::new(sine())),
        LfoWaveform::Triangle => net.push(Box::new(triangle())),
        LfoWaveform::Saw => net.push(Box::new(saw())),
    };
    net.connect(rate_id, 0, osc_id, 0);

    // Depth control
    let depth_id = net.push(Box::new(var(lfo_depth) >> follow(0.01)));

    // lfo_osc × depth
    let mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(osc_id, 0, mul_id, 0);
    net.connect(depth_id, 0, mul_id, 1);

    // dc(1.0) + (lfo × depth)
    let add_id = net.push(Box::new(An(Add2::new())));
    net.connect(dc_id, 0, add_id, 0);
    net.connect(mul_id, 0, add_id, 1);

    add_id
}

/// Build a single polyphonic voice unit with ADSR envelope, optional filter and LFO.
/// Uses an internal Net graph for dynamic node wiring.
/// Returns a 0-input, 2-output (stereo) AudioUnit.
#[allow(clippy::too_many_arguments)]
pub fn build_voice_unit(
    waveform: Waveform,
    freq: &Shared,
    gate: &Shared,
    velocity: &Shared,
    master_amp: &Shared,
    adsr: &AdsrParams,
    filter_cfg: &FilterConfig,
    cutoff: &Shared,
    resonance: &Shared,
    lfo_cfg: &LfoConfig,
    lfo_rate: &Shared,
    lfo_depth: &Shared,
) -> Box<dyn AudioUnit> {
    let mut net = Net::new(0, 2);

    // Build LFO mod node if enabled
    let lfo_mod_id = if lfo_cfg.enabled {
        Some(build_lfo_mod(&mut net, lfo_cfg, lfo_rate, lfo_depth))
    } else {
        None
    };

    // Frequency source
    let freq_id = net.push(Box::new(var(freq) >> follow(0.01)));

    // Apply LFO to frequency if targeted
    let osc_input_id = if lfo_cfg.enabled && lfo_cfg.target == LfoTarget::Frequency {
        let lfo_id = lfo_mod_id.unwrap();
        let mul_id = net.push(Box::new(An(Mul2::new())));
        net.connect(freq_id, 0, mul_id, 0);
        net.connect(lfo_id, 0, mul_id, 1);
        mul_id
    } else {
        freq_id
    };

    // Oscillator (1 input: frequency, 1 output: audio)
    let osc_id = match waveform {
        Waveform::Sine => net.push(Box::new(sine())),
        Waveform::Saw => net.push(Box::new(saw())),
        Waveform::Square => net.push(Box::new(square())),
        Waveform::Triangle => net.push(Box::new(triangle())),
    };
    net.connect(osc_input_id, 0, osc_id, 0);

    // ADSR envelope (0 input, 1 output)
    let env_id = net.push(Box::new(
        var(gate) >> adsr_live(adsr.attack, adsr.decay, adsr.sustain, adsr.release),
    ));

    // osc × envelope
    let env_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(osc_id, 0, env_mul_id, 0);
    net.connect(env_id, 0, env_mul_id, 1);

    // × velocity
    let vel_id = net.push(Box::new(var(velocity)));
    let vel_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(env_mul_id, 0, vel_mul_id, 0);
    net.connect(vel_id, 0, vel_mul_id, 1);

    let mut signal_id = vel_mul_id;

    // Optional filter (3 inputs: audio, cutoff_hz, Q → 1 output)
    if filter_cfg.enabled {
        // Cutoff source
        let cutoff_id = net.push(Box::new(var(cutoff) >> follow(0.01)));

        // Apply LFO to cutoff if targeted
        let filter_cutoff_id = if lfo_cfg.enabled && lfo_cfg.target == LfoTarget::Cutoff {
            let lfo_id = lfo_mod_id.unwrap();
            let mul_id = net.push(Box::new(An(Mul2::new())));
            net.connect(cutoff_id, 0, mul_id, 0);
            net.connect(lfo_id, 0, mul_id, 1);
            mul_id
        } else {
            cutoff_id
        };

        // Q source from resonance via var_fn
        let q_id = net.push(Box::new(var_fn(resonance, resonance_to_q)));

        // Filter node
        let filter_id = match filter_cfg.filter_type {
            FilterType::Lowpass => net.push(Box::new(lowpass())),
            FilterType::Highpass => net.push(Box::new(highpass())),
            FilterType::Bandpass => net.push(Box::new(bandpass())),
        };
        net.connect(signal_id, 0, filter_id, 0);
        net.connect(filter_cutoff_id, 0, filter_id, 1);
        net.connect(q_id, 0, filter_id, 2);

        signal_id = filter_id;
    }

    // Optional LFO on amplitude
    if lfo_cfg.enabled && lfo_cfg.target == LfoTarget::Amplitude {
        let lfo_id = lfo_mod_id.unwrap();
        let mul_id = net.push(Box::new(An(Mul2::new())));
        net.connect(signal_id, 0, mul_id, 0);
        net.connect(lfo_id, 0, mul_id, 1);
        signal_id = mul_id;
    }

    // × master amplitude
    let amp_id = net.push(Box::new(var(master_amp) >> follow(0.01)));
    let amp_mul_id = net.push(Box::new(An(Mul2::new())));
    net.connect(signal_id, 0, amp_mul_id, 0);
    net.connect(amp_id, 0, amp_mul_id, 1);

    // Split to stereo (1 input → 2 outputs)
    let split_id = net.push(Box::new(split::<U2>()));
    net.connect(amp_mul_id, 0, split_id, 0);

    // Connect to net outputs
    net.connect_output(split_id, 0, 0);
    net.connect_output(split_id, 1, 1);

    Box::new(net)
}

/// Build a polyphonic audio graph with 8 voices summed together, plus effects chain.
/// Returns the graph plus left/right Snoop frontends for oscilloscope visualization.
#[allow(clippy::too_many_arguments)]
pub fn build_poly_graph(
    waveform: Waveform,
    voices: &[Voice],
    master_amp: &Shared,
    adsr: &AdsrParams,
    filter_cfg: &FilterConfig,
    cutoff: &Shared,
    resonance: &Shared,
    lfo_cfg: &LfoConfig,
    lfo_rate: &Shared,
    lfo_depth: &Shared,
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

    // Sum points: pass() nodes where all voices feed into (Net auto-sums multiple connections)
    let sum_l_id = net.push(Box::new(pass()));
    let sum_r_id = net.push(Box::new(pass()));

    for voice in voices {
        let unit = build_voice_unit(
            waveform,
            &voice.freq,
            &voice.gate,
            &voice.velocity,
            master_amp,
            adsr,
            filter_cfg,
            cutoff,
            resonance,
            lfo_cfg,
            lfo_rate,
            lfo_depth,
        );
        let voice_id = net.push(unit);
        net.connect(voice_id, 0, sum_l_id, 0);
        net.connect(voice_id, 1, sum_r_id, 0);
    }

    // Effects chain: start from sum points
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

    // Connect effects output to snoops
    net.connect(chain_l, 0, snoop_l_id, 0);
    net.connect(chain_r, 0, snoop_r_id, 0);

    (Box::new(net), snoop_l, snoop_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f64 = 44100.0;

    fn collect_samples(mut graph: Box<dyn AudioUnit>, num_samples: usize) -> Vec<(f32, f32)> {
        graph.set_sample_rate(SAMPLE_RATE);
        graph.allocate();
        (0..num_samples).map(|_| graph.get_stereo()).collect()
    }

    fn default_filter_cfg() -> FilterConfig {
        FilterConfig::default()
    }

    fn default_lfo_cfg() -> LfoConfig {
        LfoConfig::default()
    }

    fn default_shared_params() -> (Shared, Shared, Shared, Shared) {
        (
            Shared::new(1000.0), // cutoff
            Shared::new(0.0),    // resonance
            Shared::new(1.0),    // lfo_rate
            Shared::new(0.0),    // lfo_depth
        )
    }

    fn default_effects_cfg() -> EffectsConfig {
        EffectsConfig::default()
    }

    fn default_effects_shared() -> (Shared, Shared, Shared, Shared, Shared) {
        (
            Shared::new(0.3),  // delay_time
            Shared::new(0.3),  // delay_feedback
            Shared::new(0.0),  // delay_mix
            Shared::new(0.0),  // reverb_mix
            Shared::new(0.0),  // chorus_mix
        )
    }

    #[test]
    fn build_oscillator_returns_stereo_graph() {
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let graph = build_oscillator(waveform, 440.0, 0.5);
            assert_eq!(graph.inputs(), 0, "{waveform} should have 0 inputs");
            assert_eq!(
                graph.outputs(),
                2,
                "{waveform} should have 2 outputs (stereo)"
            );
        }
    }

    #[test]
    fn all_waveforms_produce_nonsilent_output() {
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let samples = collect_samples(build_oscillator(waveform, 440.0, 0.5), 512);
            let has_nonzero = samples.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
            assert!(has_nonzero, "{waveform} should produce non-silent output");
        }
    }

    #[test]
    fn zero_amplitude_produces_silence() {
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let samples = collect_samples(build_oscillator(waveform, 440.0, 0.0), 512);
            let all_silent = samples.iter().all(|(l, r)| *l == 0.0 && *r == 0.0);
            assert!(all_silent, "{waveform} at amplitude 0.0 should be silent");
        }
    }

    #[test]
    fn output_stays_within_amplitude_bounds() {
        let amplitude = 0.5;
        // Allow small margin for filter transients
        let margin = 0.1;
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let samples = collect_samples(build_oscillator(waveform, 440.0, amplitude), 4096);
            for (i, (l, r)) in samples.iter().enumerate() {
                assert!(
                    l.abs() <= amplitude + margin,
                    "{waveform} left sample {i} out of bounds: {l}"
                );
                assert!(
                    r.abs() <= amplitude + margin,
                    "{waveform} right sample {i} out of bounds: {r}"
                );
            }
        }
    }

    #[test]
    fn stereo_channels_are_identical() {
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let samples = collect_samples(build_oscillator(waveform, 440.0, 0.5), 256);
            for (i, (l, r)) in samples.iter().enumerate() {
                assert_eq!(l, r, "{waveform} sample {i}: left ({l}) != right ({r})");
            }
        }
    }

    #[test]
    fn display_shows_lowercase_name() {
        assert_eq!(Waveform::Sine.to_string(), "sine");
        assert_eq!(Waveform::Saw.to_string(), "saw");
        assert_eq!(Waveform::Square.to_string(), "square");
        assert_eq!(Waveform::Triangle.to_string(), "triangle");
    }

    #[test]
    fn build_oscillator_shared_returns_stereo_graph() {
        let freq = Shared::new(440.0);
        let amp = Shared::new(0.5);
        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let (graph, _, _) = build_oscillator_shared(waveform, &freq, &amp);
            assert_eq!(graph.inputs(), 0, "{waveform} shared should have 0 inputs");
            assert_eq!(
                graph.outputs(),
                2,
                "{waveform} shared should have 2 outputs (stereo)"
            );
        }
    }

    #[test]
    fn build_oscillator_shared_responds_to_param_changes() {
        let freq = Shared::new(440.0);
        let amp = Shared::new(0.5);
        let (graph, _, _) = build_oscillator_shared(Waveform::Sine, &freq, &amp);
        let samples_before = collect_samples(graph, 512);
        let has_nonzero = samples_before.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "shared oscillator should produce non-silent output");

        // Change amplitude to 0 and verify output approaches silence
        amp.set_value(0.0);
        let (graph, _, _) = build_oscillator_shared(Waveform::Sine, &freq, &amp);
        // Run enough samples for follow filter to converge
        let samples_after = collect_samples(graph, 8192);
        // Last 256 samples should be near-silent
        let tail = &samples_after[samples_after.len() - 256..];
        let max_tail = tail
            .iter()
            .map(|(l, r)| l.abs().max(r.abs()))
            .fold(0.0f32, f32::max);
        assert!(
            max_tail < 0.05,
            "after setting amplitude to 0, tail should be near-silent, got max {max_tail}"
        );
    }

    #[test]
    fn build_oscillator_shared_snoop_receives_data() {
        let freq = Shared::new(440.0);
        let amp = Shared::new(0.5);
        let (graph, mut snoop_l, mut snoop_r) =
            build_oscillator_shared(Waveform::Sine, &freq, &amp);
        let _samples = collect_samples(graph, 1024);

        snoop_l.update();
        snoop_r.update();

        assert!(
            snoop_l.total() > 0,
            "left snoop should have received samples"
        );
        assert!(
            snoop_r.total() > 0,
            "right snoop should have received samples"
        );
    }

    #[test]
    fn build_voice_unit_returns_stereo() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(1.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams::default();
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();

        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let unit = build_voice_unit(
                waveform, &freq, &gate, &velocity, &master_amp, &adsr,
                &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
            );
            assert_eq!(unit.inputs(), 0, "{waveform} voice should have 0 inputs");
            assert_eq!(unit.outputs(), 2, "{waveform} voice should have 2 outputs");
        }
    }

    #[test]
    fn build_voice_unit_produces_sound_when_gate_triggered() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();

        let mut unit = build_voice_unit(
            Waveform::Sine, &freq, &gate, &velocity, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
        );
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Run some samples with gate off
        for _ in 0..100 {
            unit.get_stereo();
        }

        // Trigger the gate
        gate.set_value(1.0);

        let mut samples = Vec::new();
        for _ in 0..8192 {
            samples.push(unit.get_stereo());
        }

        let tail = &samples[4096..];
        let has_nonzero = tail.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "voice should produce sound after gate trigger");
    }

    #[test]
    fn build_voice_unit_silent_when_gate_off() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.001,
        };
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();

        let unit = build_voice_unit(
            Waveform::Sine, &freq, &gate, &velocity, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
        );
        let samples = collect_samples(unit, 4096);
        let tail = &samples[samples.len() - 256..];
        let max_tail = tail
            .iter()
            .map(|(l, r)| l.abs().max(r.abs()))
            .fold(0.0f32, f32::max);
        assert!(
            max_tail < 0.01,
            "voice with gate off should be near-silent, got {max_tail}"
        );
    }

    #[test]
    fn build_voice_unit_with_filter_returns_stereo() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(1.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams::default();
        let cutoff = Shared::new(1000.0);
        let resonance = Shared::new(0.0);
        let lfo_cfg = default_lfo_cfg();
        let lfo_rate = Shared::new(1.0);
        let lfo_depth = Shared::new(0.0);

        for filter_type in [FilterType::Lowpass, FilterType::Highpass, FilterType::Bandpass] {
            let filter_cfg = FilterConfig {
                filter_type,
                enabled: true,
            };
            let unit = build_voice_unit(
                Waveform::Saw, &freq, &gate, &velocity, &master_amp, &adsr,
                &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
            );
            assert_eq!(unit.inputs(), 0);
            assert_eq!(unit.outputs(), 2);
        }
    }

    #[test]
    fn build_voice_unit_with_filter_produces_sound() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = FilterConfig {
            filter_type: FilterType::Lowpass,
            enabled: true,
        };
        let cutoff = Shared::new(5000.0);
        let resonance = Shared::new(0.0);
        let lfo_cfg = default_lfo_cfg();
        let lfo_rate = Shared::new(1.0);
        let lfo_depth = Shared::new(0.0);

        let mut unit = build_voice_unit(
            Waveform::Saw, &freq, &gate, &velocity, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
        );
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();
        for _ in 0..100 { unit.get_stereo(); }
        gate.set_value(1.0);
        let mut samples = Vec::new();
        for _ in 0..8192 { samples.push(unit.get_stereo()); }
        let tail = &samples[4096..];
        let has_nonzero = tail.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "filtered voice should produce sound");
    }

    #[test]
    fn build_voice_unit_with_lfo_returns_stereo() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(1.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams::default();
        let filter_cfg = default_filter_cfg();
        let cutoff = Shared::new(1000.0);
        let resonance = Shared::new(0.0);
        let lfo_rate = Shared::new(5.0);
        let lfo_depth = Shared::new(0.5);

        for target in [LfoTarget::Frequency, LfoTarget::Cutoff, LfoTarget::Amplitude] {
            for waveform_lfo in [LfoWaveform::Sine, LfoWaveform::Triangle, LfoWaveform::Saw] {
                let lfo_cfg = LfoConfig {
                    waveform: waveform_lfo,
                    target,
                    enabled: true,
                };
                let unit = build_voice_unit(
                    Waveform::Saw, &freq, &gate, &velocity, &master_amp, &adsr,
                    &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
                );
                assert_eq!(unit.inputs(), 0);
                assert_eq!(unit.outputs(), 2);
            }
        }
    }

    #[test]
    fn build_voice_unit_with_lfo_on_freq_produces_sound() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = default_filter_cfg();
        let cutoff = Shared::new(1000.0);
        let resonance = Shared::new(0.0);
        let lfo_cfg = LfoConfig {
            waveform: LfoWaveform::Sine,
            target: LfoTarget::Frequency,
            enabled: true,
        };
        let lfo_rate = Shared::new(5.0);
        let lfo_depth = Shared::new(0.3);

        let mut unit = build_voice_unit(
            Waveform::Sine, &freq, &gate, &velocity, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
        );
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();
        for _ in 0..100 { unit.get_stereo(); }
        gate.set_value(1.0);
        let mut samples = Vec::new();
        for _ in 0..8192 { samples.push(unit.get_stereo()); }
        let tail = &samples[4096..];
        let has_nonzero = tail.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "voice with LFO on frequency should produce sound");
    }

    #[test]
    fn build_voice_unit_with_filter_and_lfo_on_cutoff() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0);
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = FilterConfig {
            filter_type: FilterType::Lowpass,
            enabled: true,
        };
        let cutoff = Shared::new(2000.0);
        let resonance = Shared::new(0.3);
        let lfo_cfg = LfoConfig {
            waveform: LfoWaveform::Sine,
            target: LfoTarget::Cutoff,
            enabled: true,
        };
        let lfo_rate = Shared::new(2.0);
        let lfo_depth = Shared::new(0.5);

        let mut unit = build_voice_unit(
            Waveform::Saw, &freq, &gate, &velocity, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
        );
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();
        for _ in 0..100 { unit.get_stereo(); }
        gate.set_value(1.0);
        let mut samples = Vec::new();
        for _ in 0..8192 { samples.push(unit.get_stereo()); }
        let tail = &samples[4096..];
        let has_nonzero = tail.iter().any(|(l, r)| *l != 0.0 || *r != 0.0);
        assert!(has_nonzero, "voice with filter + LFO on cutoff should produce sound");
    }

    #[test]
    fn build_poly_graph_returns_stereo() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams::default();
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();
        let effects_cfg = default_effects_cfg();
        let (delay_time, delay_feedback, delay_mix, reverb_mix, chorus_mix) =
            default_effects_shared();

        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let (graph, _, _) = build_poly_graph(
                waveform, &voices, &master_amp, &adsr,
                &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
                &effects_cfg, &delay_time, &delay_feedback, &delay_mix,
                &reverb_mix, &chorus_mix,
            );
            assert_eq!(graph.inputs(), 0, "{waveform} poly should have 0 inputs");
            assert_eq!(graph.outputs(), 2, "{waveform} poly should have 2 outputs");
        }
    }

    #[test]
    fn build_poly_graph_snoop_receives_data() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        voices[0].freq.set_value(440.0);
        voices[0].gate.set_value(1.0);
        voices[0].velocity.set_value(1.0);

        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();
        let effects_cfg = default_effects_cfg();
        let (delay_time, delay_feedback, delay_mix, reverb_mix, chorus_mix) =
            default_effects_shared();

        let (graph, mut snoop_l, mut snoop_r) = build_poly_graph(
            Waveform::Sine, &voices, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
            &effects_cfg, &delay_time, &delay_feedback, &delay_mix,
            &reverb_mix, &chorus_mix,
        );
        let _samples = collect_samples(graph, 2048);

        snoop_l.update();
        snoop_r.update();

        assert!(snoop_l.total() > 0, "left snoop should have received data");
        assert!(snoop_r.total() > 0, "right snoop should have received data");
    }

    #[test]
    fn build_poly_graph_with_effects_enabled() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        voices[0].freq.set_value(440.0);
        voices[0].gate.set_value(1.0);
        voices[0].velocity.set_value(1.0);

        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };
        let filter_cfg = default_filter_cfg();
        let lfo_cfg = default_lfo_cfg();
        let (cutoff, resonance, lfo_rate, lfo_depth) = default_shared_params();
        let effects_cfg = EffectsConfig {
            delay_enabled: true,
            reverb_enabled: true,
            chorus_enabled: true,
            ..EffectsConfig::default()
        };
        let delay_time = Shared::new(0.1);
        let delay_feedback = Shared::new(0.3);
        let delay_mix = Shared::new(0.3);
        let reverb_mix = Shared::new(0.3);
        let chorus_mix = Shared::new(0.3);

        let (graph, mut snoop_l, mut snoop_r) = build_poly_graph(
            Waveform::Sine, &voices, &master_amp, &adsr,
            &filter_cfg, &cutoff, &resonance, &lfo_cfg, &lfo_rate, &lfo_depth,
            &effects_cfg, &delay_time, &delay_feedback, &delay_mix,
            &reverb_mix, &chorus_mix,
        );
        let _samples = collect_samples(graph, 2048);

        snoop_l.update();
        snoop_r.update();

        assert!(snoop_l.total() > 0, "snoop should receive data with effects");
        assert!(snoop_r.total() > 0, "snoop should receive data with effects");
    }

    #[test]
    fn adsr_params_default_values() {
        let adsr = AdsrParams::default();
        assert_eq!(adsr.attack, 0.01);
        assert_eq!(adsr.decay, 0.1);
        assert_eq!(adsr.sustain, 0.7);
        assert_eq!(adsr.release, 0.3);
    }
}
