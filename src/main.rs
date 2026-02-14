mod engine;

use clap::Parser;
use engine::oscillator::Waveform;

#[derive(Parser)]
#[command(name = "synthesis", about = "A Rust audio synthesizer")]
struct Cli {
    /// Waveform type
    #[arg(short, long, default_value = "sine")]
    waveform: Waveform,

    /// Frequency in Hz
    #[arg(short, long, default_value_t = 440.0)]
    frequency: f32,

    /// Amplitude (0.0 to 1.0)
    #[arg(short, long, default_value_t = 0.5)]
    amplitude: f32,

    /// Duration in seconds
    #[arg(short, long, default_value_t = 5.0)]
    duration: f32,
}

fn main() {
    let cli = Cli::parse();
    engine::play(cli.waveform, cli.frequency, cli.amplitude, cli.duration);
}
