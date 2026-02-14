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
