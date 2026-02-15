# synthesis

A real-time polyphonic audio synthesizer written in Rust with a Synthwave-themed UI.

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

### Options (CLI standalone mode)

| Option | Description | Default |
|---|---|---|
| `-w, --waveform` | Waveform type: `sine`, `saw`, `square`, `triangle` | `sine` |
| `-f, --frequency` | Frequency in Hz | `440` |
| `-a, --amplitude` | Amplitude (0.0 to 1.0) | `0.5` |
| `-d, --duration` | Duration in seconds | `5` |
| `--gui` | Launch the Synthwave mixer GUI (per-voice controls, MIDI, keyboard) | off |
| `--tenori` | Launch the Tenori-on sequencer interface | off |

> In GUI mode (`--gui`), all parameters are controlled per-voice through the 8 channel strips. CLI options are for standalone single-tone playback only.

### Examples

```bash
# Play a 440 Hz sine wave for 5 seconds
cargo run --release

# Play a sawtooth wave at 261.63 Hz (middle C) for 3 seconds
cargo run --release -- --waveform saw --frequency 261.63 --duration 3

# Play a quiet square wave
cargo run --release -- -w square -a 0.2

# Launch the Synthwave GUI
cargo run --release -- --gui

# Launch the Tenori-on sequencer
cargo run --release -- --tenori
```

## GUI features (`--gui`)

- **Synthwave dark purple theme** with custom widgets (knobs, sliders, VU meters)
- **8 per-voice channel strips** (mixer-style): each voice has independent waveform, ADSR, filter, LFO, and level
- **Test button per voice**: plays C4 through the specific voice with its own settings
- **Waveform selector per voice**: sine, saw, square, triangle (compact toggle buttons)
- **ADSR envelope per voice**: attack, decay, sustain, release mini knobs
- **Resonant filter per voice**: lowpass, highpass, bandpass with cutoff and resonance, enable/disable toggle
- **LFO per voice**: modulation targeting frequency, cutoff, or amplitude with rate and depth, enable/disable toggle
- **Per-voice level**: individual volume control per voice
- **Master volume** vertical slider with stereo VU meters
- **Effects** (global post-mix): delay (time, feedback, mix), reverb (room size, time, mix), chorus (separation, variation, mod freq, mix)
- **Configurable effects chain**: reorderable slots, per-effect enable/bypass
- **Presets**: 5 factory presets (Init, Warm Pad, Sharp Lead, Deep Bass, Space FX), save/load user presets
- **MIDI input**: connect to any MIDI controller
- **Virtual keyboard**: 2-octave piano (C3-B4) with mouse interaction and key highlight
- **Keyboard shortcuts**: AZERTY piano mapping, Ctrl+1-4 (waveform all voices), Space (panic), Tab (filter cycle)
- **8-voice polyphony** with per-strip voice activity LEDs (green/yellow/gray)
- **Oscilloscope**: real-time waveform display in master panel
- **Drum pads**: 4x4 visual grid

## Tenori-on sequencer (`--tenori`)

A 16x16 matrix sequencer inspired by the Yamaha Tenori-on, with real-time audio synthesis.

### Lead mode (melodic)

- **16x16 grid matrix**: rows = notes (C5 to A3), columns = time steps
- **Playhead**: advances left-to-right, triggering active cells as notes
- **Scale selector**: Chromatic, Major, Minor, Pentatonic
- **Synth parameters** (sidebar): oscillator (waveform, pitch, detune), ADSR envelope, resonant filter (LP/HP/BP), LFO (pitch/filter/amp modulation), effects (reverb, delay, chorus)

### Drummer mode (percussion)

- **16-instrument drum kit**: rows = percussion instruments (Crash, Ride, Open HH, Closed HH, Clap, Rimshot, Snare, Tom Hi, Tom Mid, Tom Low, Conga Hi, Conga Lo, Cowbell, Claves, Kick, Kick Hard)
- **Fixed-topology dual-source synthesis**: noise + sine mixed per-hit via atomic parameters — zero graph rebuilds
- **Drum Kit panel** (sidebar): Tune (pitch offset), Decay, Color (filter brightness)
- **Shared effects**: delay, reverb, chorus work in both modes

### Common features

- **Mode toggle**: switch Lead/Drummer with M key or transport button (independent grids preserved)
- **Draw modes**: Toggle, Draw, Erase (switch with keyboard shortcuts D/E/T)
- **Transport controls**: play/pause, BPM slider (40-240), swing knob
- **Custom widgets**: rotary knobs, horizontal sliders, toggle button groups
- **Density bar**: visual indicator of note density per column
- **Active notes display**: header shows currently playing note names
- **Undo/redo**: Ctrl+Z / Ctrl+Shift+Z (20-level circular history)
- **Modifier clicks**: Shift+click toggles row, Ctrl+click toggles column
- **8-voice polyphony** with voice stealing

### Tenori-on keyboard shortcuts

| Shortcut | Action |
|---|---|
| Space | Toggle play/pause |
| M | Toggle Lead/Drummer mode |
| C | Clear grid |
| D | Draw mode |
| E | Erase mode |
| T | Toggle mode |
| Ctrl+Up/Down | BPM +/- 5 |
| Ctrl+1/2/3/4 | Waveform: Sine/Saw/Square/Tri |
| Tab | Cycle filter type (LP/HP/BP) |
| Shift+Tab | Cycle LFO target |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z / Ctrl+Y | Redo |
| Shift+click | Toggle entire row |
| Ctrl+click | Toggle entire column |

## Mixer keyboard shortcuts

| Shortcut | Action |
|---|---|
| W, X, C, V, B, N, Comma | Piano octave 3 (white keys C3-B3) |
| S, D, G, H, J | Piano octave 3 (black keys) |
| A, Z, E, R, T, Y, U | Piano octave 4 (white keys C4-B4) |
| 2, 3, 5, 6, 7 | Piano octave 4 (black keys) |
| Ctrl+1/2/3/4 | Select waveform on all voices (Sine/Saw/Square/Triangle) |
| Ctrl+Up/Down | Volume +/- 0.05 |
| Tab | Cycle filter type (LP/HP/BP) |
| Shift+Tab | Cycle LFO target |
| Space | Panic (all notes off) |

## Project structure

```
src/
├── main.rs              # CLI entry point (clap), --gui/--tenori flags
├── midi.rs              # MIDI input handler (midir), NoteEvent, TestOn/TestOff
├── preset.rs            # Preset system: save/load JSON, factory presets
├── engine/
│   ├── mod.rs           # Audio output (cpal), init/start helpers
│   ├── oscillator.rs    # Waveform generation (fundsp), ADSR, filter, LFO, effects, poly graph
│   ├── drum.rs          # Drum synthesis: DrumParams, DRUM_KIT, DrumVoiceShared, drum poly graph
│   ├── tenori.rs        # Combined Tenori graph: 8 lead + 8 drum voices, shared effects
│   ├── filter.rs        # FilterConfig, LfoConfig, Mul2, Add2, resonance_to_q
│   ├── effects.rs       # FeedbackDelay, EffectsConfig, wire_delay/reverb/chorus
│   └── voice.rs         # Voice, VoiceAllocator, VoiceConfig, VoiceShared
├── gui/
│   ├── mod.rs           # GUI entry point (eframe), run() + run_tenori()
│   ├── app.rs           # SynthApp: per-voice config sync, audio engine bridge
│   ├── tenori_app.rs    # TenoriApp: tenori-on UI + audio engine bridge
│   └── oscilloscope.rs  # Waveform visualizer (egui_plot)
├── synth_ui/
│   ├── mod.rs           # SynthUI: 8-strip layout, keyboard shortcuts, params bridge
│   ├── theme.rs         # SynthTheme: Synthwave color palette, panel frame
│   ├── widgets/
│   │   ├── knob.rs      # Rotary knob (58×78) + mini knob (36×52) for strips
│   │   ├── vslider.rs   # Vertical slider (volume)
│   │   ├── hslider.rs   # Horizontal slider (filter, effects, level)
│   │   ├── level_meter.rs # VU meter (15 segments)
│   │   ├── select_buttons.rs # Toggle button group
│   │   ├── keyboard.rs  # Piano keyboard (2 octaves, themed)
│   │   └── pads.rs      # Drum pads (4x4 grid)
│   └── panels/
│       ├── voice_strip.rs   # Per-voice channel strip
│       ├── effects.rs       # Delay/reverb/chorus controls
│       ├── master.rs        # Volume + VU meters + oscilloscope
│       ├── keyboard_panel.rs # Piano keyboard wrapper
│       └── pads_panel.rs    # Drum pads wrapper
└── tenori_synth/
    ├── mod.rs           # TenoriSynth: main struct, layout, playhead logic
    ├── state.rs         # TenoriState: grid, enums, scale, MIDI mapping
    ├── theme.rs         # Theme: colors, dimensions, shadows
    ├── grid.rs          # 16×16 matrix rendering and interaction
    ├── transport.rs     # Play/pause, BPM, swing controls
    ├── density_bar.rs   # Per-column density visualization
    ├── header.rs        # Title, LED, active notes display
    ├── shortcuts.rs     # Keyboard shortcut handling
    ├── history.rs       # Undo/redo system (20-entry circular)
    ├── widgets/
    │   ├── knob.rs      # Rotary knob (configurable size, drag, double-click reset)
    │   ├── hslider.rs   # Horizontal slider with label + value
    │   ├── select_buttons.rs # Toggle button group
    │   └── panel.rs     # Panel wrapper with title
    └── panels/
        ├── oscillator.rs # Waveform + pitch + detune
        ├── envelope.rs   # ADSR knobs
        ├── filter.rs     # LP/HP/BP + cutoff + resonance
        ├── lfo.rs        # Rate + depth + target
        ├── effects.rs    # Reverb/delay/chorus sliders
        ├── scale.rs      # Scale selector
        ├── draw_mode.rs  # Draw mode selector
        └── drum_kit.rs   # Drum Kit panel: Tune, Decay, Color
```

## Test

```bash
cargo test
```

## License

MIT — see [LICENSE](LICENSE) for details.
