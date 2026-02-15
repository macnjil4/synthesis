mod engine;
mod gui;
mod midi;
mod preset;
mod synth_ui;
mod tenori_synth;

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

    /// Launch the graphical user interface
    #[arg(long)]
    gui: bool,

    /// Launch the Tenori-on sequencer interface
    #[arg(long)]
    tenori: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.tenori {
        gui::run_tenori();
    } else if cli.gui {
        gui::run();
    } else {
        engine::play(cli.waveform, cli.frequency, cli.amplitude, cli.duration);
    }
}
