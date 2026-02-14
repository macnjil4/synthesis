# synthesis

A real-time polyphonic audio synthesizer written in Rust.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)
- macOS with audio output

## Build

```bash
cargo build --release
```

> Audio code must be compiled in release mode to avoid glitches.

## Usage

```bash
cargo run --release -- [OPTIONS]
```

### Options

| Option | Description | Default |
|---|---|---|
| `-w, --waveform` | Waveform type: `sine`, `saw`, `square`, `triangle` | `sine` |
| `-f, --frequency` | Frequency in Hz | `440` |
| `-a, --amplitude` | Amplitude (0.0 to 1.0) | `0.5` |
| `-d, --duration` | Duration in seconds | `5` |
| `--gui` | Launch the graphical user interface | off |

### Examples

```bash
# Play a 440 Hz sine wave for 5 seconds
cargo run --release

# Play a sawtooth wave at 261.63 Hz (middle C) for 3 seconds
cargo run --release -- --waveform saw --frequency 261.63 --duration 3

# Play a quiet square wave
cargo run --release -- -w square -a 0.2

# Launch the GUI (polyphony, ADSR, MIDI, virtual keyboard)
cargo run --release -- --gui
```

## GUI features

- **Waveform selector**: sine, saw, square, triangle
- **Master volume** slider
- **ADSR envelope**: attack, decay, sustain, release sliders
- **MIDI input**: connect to any MIDI controller
- **Virtual keyboard**: 2-octave piano (C3–B4) with mouse interaction
- **8-voice polyphony** with voice activity indicators
- **Oscilloscope**: real-time waveform display

## Project structure

```
src/
├── main.rs              # CLI entry point (clap), --gui flag
├── midi.rs              # MIDI input handler (midir), NoteEvent
├── engine/
│   ├── mod.rs           # Audio output (cpal), init/start helpers
│   ├── oscillator.rs    # Waveform generation (fundsp), ADSR, poly graph
│   └── voice.rs         # Voice, VoiceAllocator, midi_note_to_freq
└── gui/
    ├── mod.rs           # GUI entry point (eframe)
    ├── app.rs           # SynthApp: polyphony, MIDI, ADSR controls
    ├── keyboard.rs      # Virtual piano keyboard widget
    └── oscilloscope.rs  # Waveform visualizer (egui_plot)
```

## Test

```bash
cargo test
```

## License

MIT — see [LICENSE](LICENSE) for details.
