# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Unit tests for oscillator module (stereo output, non-silence, zero amplitude, amplitude bounds, channel identity, display names)
- Integration tests for CLI (help flag, invalid waveform rejection)

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
