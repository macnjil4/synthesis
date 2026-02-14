use fundsp::prelude32::*;

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
}
