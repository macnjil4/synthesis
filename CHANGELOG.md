# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-02-14

### Added

- Resonant filter with 3 modes: lowpass (LP), highpass (HP), bandpass (BP)
- Filter cutoff (20–20000 Hz, logarithmic) and resonance (0.0–1.0) sliders
- LFO (low-frequency oscillator) with 3 waveforms: sine, triangle, saw
- LFO modulation targets: frequency, filter cutoff, amplitude
- LFO rate (0.1–20 Hz) and depth (0.0–1.0) sliders
- Filter and LFO enable/disable checkboxes with live toggle
- `FilterConfig`, `LfoConfig` types in `src/engine/filter.rs`
- Custom `Mul2` and `Add2` AudioNode implementations for Net graph wiring
- `resonance_to_q()` mapping function (0.0→0.5, 1.0→20.0)
- `build_lfo_mod()` helper for LFO modulation graph construction
- Unit tests for filter types, LFO combinations, and filter+LFO integration

### Changed

- `build_voice_unit()` rewritten to use internal `Net` graph (dynamic node wiring)
- `build_poly_graph()` extended with filter and LFO parameters
- Rebuild detection includes filter config and LFO config changes
- Continuous parameters (cutoff, resonance, LFO rate/depth) via `Shared` (no rebuild)
- Window size increased to 800×850 to accommodate filter and LFO controls

## [0.3.0] - 2026-02-14

### Added

- ADSR envelope per voice (`adsr_live`): attack, decay, sustain, release sliders
- 8-voice polyphony with voice allocator (idle → releasing → round-robin stealing)
- MIDI input via `midir`: port selection, connect/disconnect, note on/off
- Virtual piano keyboard (C3–B4, 2 octaves) with mouse interaction
- Voice activity indicators (green = playing, yellow = releasing, gray = idle)
- Polyphonic audio graph using `fundsp::Net` (8 voices summed to stereo output)
- `VoiceAllocator` module (`src/engine/voice.rs`): voice allocation, `midi_note_to_freq()`
- `MidiHandler` module (`src/midi.rs`): MIDI parsing, port management, mpsc channel
- `build_voice_unit()` and `build_poly_graph()` in oscillator module
- Unit tests: voice allocation, MIDI parsing, ADSR voice unit, poly graph
- Dependency: `midir` 0.10

### Changed

- GUI reworked: frequency slider replaced by ADSR sliders and keyboard
- Audio graph rebuilt on waveform or ADSR parameter change
- Window size increased to 700x700 to accommodate new controls

### Removed

- Single-voice frequency slider (frequency now comes from note events)

## [0.2.0] - 2026-02-14

### Added

- GUI mode via `--gui` flag: egui/eframe window with real-time audio controls
- Waveform selector (radio buttons: sine, saw, square, triangle)
- Frequency slider (20 Hz – 20 kHz, logarithmic scale)
- Volume slider (0.0 – 1.0)
- Play/stop button with live stream management
- Oscilloscope visualization using `egui_plot` (left + right channels)
- Lock-free parameter control via `fundsp::Shared` (atomic floats)
- Audio snoop ring buffers (`fundsp::Snoop`) for waveform display
- Engine helpers: `init_audio_device()`, `start_stream()`, `build_oscillator_shared()`
- Unit tests for `build_oscillator_shared` (stereo output, parameter response, snoop data)
- Integration test for `--gui` flag in CLI help
- Dependencies: `eframe` 0.33, `egui_plot` 0.34

### Changed

- `Waveform` enum now derives `PartialEq` (needed for waveform change detection)

## [0.1.0] - 2026-02-14

### Added

- Audio engine with real-time output via `cpal` (44100 Hz, stereo, 256-sample buffer)
- 4 oscillator waveforms via `fundsp`: sine, saw, square, triangle
- CLI interface with `clap`: `--waveform`, `--frequency`, `--amplitude`, `--duration`
- Engine module structure (`src/engine/`)
- Initial project setup with `cargo init`
- MIT license
- README with build/run/test instructions
- CHANGELOG following Keep a Changelog format
