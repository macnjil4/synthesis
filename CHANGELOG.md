# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

## [0.2.0] - 2026-02-14

### Added

- Audio engine with real-time output via `cpal` (44100 Hz, stereo, 256-sample buffer)
- 4 oscillator waveforms via `fundsp`: sine, saw, square, triangle
- CLI interface with `clap`: `--waveform`, `--frequency`, `--amplitude`, `--duration`
- Engine module structure (`src/engine/`)

## [0.1.0] - 2026-02-14

### Added

- Initial project setup with `cargo init`
- MIT license
- README with build/run/test instructions
- CHANGELOG following Keep a Changelog format
