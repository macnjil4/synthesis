# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-02-15

### Added

- Per-voice channel strip architecture: each of the 8 voices has independent waveform, ADSR, filter, LFO, and level
- `VoiceConfig` struct for per-voice topology and runtime parameters
- `VoiceShared` struct for per-voice atomic shared parameters (cutoff, resonance, LFO rate/depth, level)
- Per-voice level control (`voice_level`) in the audio signal chain
- Channel strip UI: 8 compact vertical strips displayed side-by-side (mixer-style layout)
- Mini knob widget (36x52px) for compact channel strip ADSR controls
- Test button per voice: plays C4 through the specific voice with its own settings (~500ms)
- `force_note_on()` / `force_note_off()` on `VoiceAllocator` for targeting specific voices
- `NoteEvent::TestOn` / `NoteEvent::TestOff` variants for per-voice test triggers
- `VoiceConfig::topology_differs()` for efficient rebuild detection across all 8 voices
- Filter and LFO enable/disable toggle buttons per voice strip (compact "F" and "L" buttons)
- Voice activity LED per strip (green = active, yellow = releasing, gray = idle)

### Changed

- `build_voice_unit()` now accepts per-voice parameters instead of global ones
- `build_poly_graph()` iterates with per-voice `VoiceConfig` and `VoiceShared` arrays
- `SynthApp` stores `Vec<VoiceConfig>` (8) and `Vec<VoiceShared>` (8) instead of global synth params
- `SynthParams` carries `&mut [VoiceConfig]` instead of individual global fields
- Rebuild detection checks topology changes across all 8 voice configs
- Presets apply to all 8 voices on load (retro-compatible); save uses voice 1 config
- Keyboard shortcuts (Ctrl+1-4 waveform, Tab filter, Shift+Tab LFO) now affect all voices
- Window size increased from 1100x700 to 1400x850 for 8-strip layout
- Master panel and effects moved to a dedicated column alongside voice strips
- Voice allocator prefers releasing voices over idle (faster re-trigger response)

## [0.6.0] - 2026-02-14

### Added

- Synthwave dark purple UI theme (`SynthTheme`) with custom color palette
- Custom rotary knob widget (270° arc, drag-to-adjust, value display)
- Custom vertical slider widget for ADSR and volume
- Custom horizontal slider widget for filter and effects parameters
- Select buttons widget (toggle group) for waveform, filter type, LFO selectors
- VU meter widget (15 segments: green/yellow/red) driven by snoop RMS data
- Themed piano keyboard with note names, rounded bottom corners, key highlight
- Drum pads 4×4 grid (visual only, not connected to audio)
- Panel layout system with themed frames and shadow
- Keyboard shortcuts: AZERTY piano mapping (W/X/C/V... for octave 3, A/Z/E/R... for octave 4)
- Global shortcuts: Ctrl+1-4 (waveform), Ctrl+Up/Down (volume), Tab (filter cycle), Shift+Tab (LFO target), Space (panic)
- Multi-key support via `pressed_keys: HashSet<u8>` for chords
- Pitch and Detune knobs in oscillator panel (UI-only, not connected to engine)
- Header bar with LED status indicator and version display
- Preset bar with MIDI connection controls integrated
- `SynthUI` module (`src/synth_ui/`) with 18 files: theme, 7 widgets, 8 panels, mod.rs

### Changed

- GUI completely redesigned from standard egui widgets to custom Synthwave theme
- Window size changed from 900×950 to 1100×700 (fixed layout, no scroll)
- Rendering delegated from `SynthApp::update()` to `SynthUI::show()` via `SynthParams` bridge
- Auto-start audio on launch (removed Play/Stop button)
- Old `gui/keyboard.rs` replaced by `synth_ui/widgets/keyboard.rs`
- Oscilloscope moved into Master panel
- Window title changed to "Synthwave"

## [0.5.0] - 2026-02-14

### Added

- 3 audio effects: delay (time + feedback), reverb (room size + time), chorus (separation + variation + mod freq)
- Configurable effects chain with reorderable slots (click to swap)
- Dry/wet mix control per effect via `Shared` (no rebuild needed)
- Custom `FeedbackDelay` AudioNode with ring buffer for delay with feedback
- `wire_delay()`, `wire_reverb()`, `wire_chorus()` graph wiring helpers
- `EffectSlot`, `EffectsConfig` types in `src/engine/effects.rs`
- Preset system with JSON serialization (`serde`/`serde_json`)
- Save/load user presets to `~/.synthesis/presets/`
- 5 factory presets: Init, Warm Pad, Sharp Lead, Deep Bass, Space FX
- `Preset` module (`src/preset.rs`): save, load, list, factory presets
- Preset selector dropdown and save-as text input in GUI
- Unit tests for effects, presets, and effects-enabled poly graph
- Dependencies: `serde` 1 (with derive), `serde_json` 1, `dirs` 6

### Changed

- `build_poly_graph()` extended with effects chain between voice sum and snoops
- Voices summed through `pass()` nodes for effects insertion point
- Effects enable/disable, order, and compile-time params trigger graph rebuild
- All synth parameter types derive `Serialize`/`Deserialize`
- GUI wrapped in `ScrollArea` for vertical scrolling
- Window size increased to 900x950 to accommodate effects and presets

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
