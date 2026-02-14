use fundsp::prelude32::*;

use super::voice::Voice;

/// ADSR envelope parameters.
#[derive(Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
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

/// Build a single polyphonic voice unit with ADSR envelope.
/// Returns a 0-input, 2-output (stereo) AudioUnit.
pub fn build_voice_unit(
    waveform: Waveform,
    freq: &Shared,
    gate: &Shared,
    velocity: &Shared,
    master_amp: &Shared,
    adsr: &AdsrParams,
) -> Box<dyn AudioUnit> {
    let freq_control = var(freq) >> follow(0.01);
    let envelope = var(gate) >> adsr_live(adsr.attack, adsr.decay, adsr.sustain, adsr.release);
    let vel = var(velocity);
    let amp_control = var(master_amp) >> follow(0.01);

    match waveform {
        Waveform::Sine => Box::new(
            ((freq_control >> sine()) * envelope * vel * amp_control) >> split::<U2>(),
        ),
        Waveform::Saw => Box::new(
            ((freq_control >> saw()) * envelope * vel * amp_control) >> split::<U2>(),
        ),
        Waveform::Square => Box::new(
            ((freq_control >> square()) * envelope * vel * amp_control) >> split::<U2>(),
        ),
        Waveform::Triangle => Box::new(
            ((freq_control >> triangle()) * envelope * vel * amp_control) >> split::<U2>(),
        ),
    }
}

/// Build a polyphonic audio graph with 8 voices summed together.
/// Returns the graph plus left/right Snoop frontends for oscilloscope visualization.
pub fn build_poly_graph(
    waveform: Waveform,
    voices: &[Voice],
    master_amp: &Shared,
    adsr: &AdsrParams,
) -> (Box<dyn AudioUnit>, Snoop, Snoop) {
    let mut net = Net::new(0, 2);

    let (snoop_l, snoop_backend_l) = snoop(32768);
    let (snoop_r, snoop_backend_r) = snoop(32768);

    let snoop_l_id = net.push(Box::new(snoop_backend_l));
    let snoop_r_id = net.push(Box::new(snoop_backend_r));

    net.connect_output(snoop_l_id, 0, 0);
    net.connect_output(snoop_r_id, 0, 1);

    for voice in voices {
        let unit = build_voice_unit(
            waveform,
            &voice.freq,
            &voice.gate,
            &voice.velocity,
            master_amp,
            adsr,
        );
        let voice_id = net.push(unit);
        net.connect(voice_id, 0, snoop_l_id, 0);
        net.connect(voice_id, 1, snoop_r_id, 0);
    }

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

        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let unit = build_voice_unit(waveform, &freq, &gate, &velocity, &master_amp, &adsr);
            assert_eq!(unit.inputs(), 0, "{waveform} voice should have 0 inputs");
            assert_eq!(unit.outputs(), 2, "{waveform} voice should have 2 outputs");
        }
    }

    #[test]
    fn build_voice_unit_produces_sound_when_gate_triggered() {
        let freq = Shared::new(440.0);
        let gate = Shared::new(0.0); // start with gate off
        let velocity = Shared::new(1.0);
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 0.01,
        };

        let mut unit = build_voice_unit(Waveform::Sine, &freq, &gate, &velocity, &master_amp, &adsr);
        unit.set_sample_rate(SAMPLE_RATE);
        unit.allocate();

        // Run some samples with gate off
        for _ in 0..100 {
            unit.get_stereo();
        }

        // Trigger the gate (simulates note_on)
        gate.set_value(1.0);

        // Run enough samples for ADSR attack + filter convergence
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

        let unit = build_voice_unit(Waveform::Sine, &freq, &gate, &velocity, &master_amp, &adsr);
        // With gate=0 and very short release, output should stay silent
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
    fn build_poly_graph_returns_stereo() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        let master_amp = Shared::new(0.5);
        let adsr = AdsrParams::default();

        for waveform in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let (graph, _, _) = build_poly_graph(waveform, &voices, &master_amp, &adsr);
            assert_eq!(graph.inputs(), 0, "{waveform} poly should have 0 inputs");
            assert_eq!(graph.outputs(), 2, "{waveform} poly should have 2 outputs");
        }
    }

    #[test]
    fn build_poly_graph_snoop_receives_data() {
        let voices: Vec<Voice> = (0..8).map(|_| Voice::new()).collect();
        // Activate one voice
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

        let (graph, mut snoop_l, mut snoop_r) =
            build_poly_graph(Waveform::Sine, &voices, &master_amp, &adsr);
        let _samples = collect_samples(graph, 2048);

        snoop_l.update();
        snoop_r.update();

        assert!(snoop_l.total() > 0, "left snoop should have received data");
        assert!(snoop_r.total() > 0, "right snoop should have received data");
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
