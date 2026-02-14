use fundsp::prelude32::*;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
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
}
