# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.0] - 2026-02-15

### Added

- **Sample-based drum engine**: WAV playback replaces noise+sine synthesis for realistic drum sounds
- `SamplePlayer` custom AudioNode: one-shot WAV playback with linear interpolation and pitch shifting
- `SampleDrumVoiceShared`: per-voice atomic shared parameters (sample_index, pitch_ratio, level)
- `build_sample_drum_voice_unit()`: signal chain SamplePlayer → velocity → level → master_amp → stereo
- `load_drum_kit()`: loads 16 WAV files per kit using `hound`, normalizes to f32, resamples to output rate
- 3 Oramics drum kits (Public Domain samples from [oramics/sampled](https://github.com/oramics/sampled)):
  - **LinnDrum** (LM-2): classic 80s kit, polyvalent
  - **TR-505**: Roland electronic kit
  - **CR-78**: vintage Roland, warm tones
- `DrumPreset::dir_name()` method for kit directory lookup
- 48 WAV sample files in `samples/` (16 per kit: crash, ride, hi-hats, clap, rimshot, snare, toms, congas, cowbell, clave, kick)
- 8 new unit tests for SamplePlayer, sample drum voice, and kit loading
- Dependency: `hound = "3.5"` for WAV file reading

### Changed

- `DrumPreset` enum: LinnDrum / TR-505 / CR-78 (replaces Standard/Rock/Jazz/Dance/Electronic/Latin)
- Drum voice graph uses `SamplePlayer` instead of noise+sine→filter→ADSR synthesis
- `build_matrix_graph()` accepts `drum_buffers: &Arc<Vec<Vec<f32>>>` for sample data
- Drum preset change triggers sample reload + graph rebuild (new buffers)
- Tune knob now controls pitch_ratio (0.5x–2.0x playback speed) instead of frequency offset
- Drum Kit panel dynamically shows 3 kit buttons instead of 6
- Version bumped to 0.13.0

## [0.12.0] - 2026-02-15

### Added

- **Drum kit presets**: 6 selectable drum kits — Standard, Rock, Jazz, Dance, Electronic, Latin
- `DrumPreset` enum in state with `drum_preset` field (default: Standard)
- 5 new drum kit parameter arrays: `DRUM_KIT_ROCK`, `DRUM_KIT_JAZZ`, `DRUM_KIT_DANCE`, `DRUM_KIT_ELECTRONIC`, `DRUM_KIT_LATIN`
- `drum_kit_for_preset()` function to select kit by preset
- Drum Kit panel now shows preset selector buttons above Tune/Decay/Color knobs
- No graph rebuild needed when switching drum presets (parameters set via Shared atomics per-hit)
- 5 new unit tests for drum preset state and kit lookup

### Changed

- Drum note triggering uses selected preset's kit instead of hardcoded `DRUM_KIT`
- Version bumped to 0.12.0

## [0.11.0] - 2026-02-15

### Added

- **Bass mode**: third simultaneous voice channel with pink/rose color scheme (A1→C3 range)
- 6 bass presets: Sub Bass, Acid Bass, Funk Bass, Warm Bass, Pluck Bass, Growl Bass — each with unique waveform, ADSR, filter, and LFO settings
- `BassPreset` enum in state with `bass_grid`, `bass_preset`, `BASS_NOTE_LABELS`, `BASS_BASE_MIDI_NOTE`, `row_to_bass_midi()`
- Bass preset selector panel in sidebar (Bass mode)
- Pink theme colors: `BASS_ACCENT`, `BASS_CELL_ON`, `BASS_CELL_HIT`, `BASS_CELL_HIT_GLOW`, `GHOST_BASS`
- Ghost overlay now shows 2 other modes (3-way: Lead/Drum/Bass all visible in transparency)
- 8 bass voice units in audio graph (total: 24 voices — 8 lead + 8 drum + 8 bass)
- Bass preset change triggers audio graph rebuild (topology-changing: waveform, filter, LFO)
- Scale selection (Chromatic/Major/Minor/Pentatonic) applies to Bass mode
- 11 new unit tests for Bass state, preset, grid, MIDI mapping

### Changed

- Mode cycle (M key): Lead → Drummer → Bass → Lead
- `ChannelMode::ALL` now contains 3 modes
- `build_matrix_graph()` accepts bass voices/configs/shared parameters
- Header badge shows "BASS" in pink for Bass mode
- Density bar uses pink color in Bass mode
- Version bumped to 0.11.0

## [0.10.0] - 2026-02-15

### Added

- **Simultaneous Lead + Drum playback**: both grids play at the same time through a single combined audio graph (8 lead + 8 drum voices summed together with shared effects chain)
- Combined graph builder `build_matrix_graph()` in `src/engine/matrix.rs`: 16 voice units in one Net graph, single effects chain
- **Ghost overlay**: when editing one mode, the other mode's active cells are visible as semi-transparent background (yellow for Lead, purple for Drum)
- **Lead mode yellow color scheme**: Lead cells are now gold/yellow instead of purple, giving clear visual distinction between modes
- Ghost overlay colors in theme: `GHOST_LEAD` (transparent yellow), `GHOST_DRUM` (transparent purple)
- Lead-specific cell colors: `LEAD_CELL_ON`, `LEAD_CELL_ON_HOVER`, `LEAD_CELL_HIT`, `LEAD_CELL_HIT_GLOW`, `LEAD_ACCENT_*`
- 2 new unit tests for the combined graph builder

### Changed

- Mode switching no longer triggers audio graph rebuild (both graphs always active)
- `handle_step_change()` triggers both lead notes and drum hits simultaneously on every step
- `sync_voice_configs_from_matrix()` always syncs lead parameters regardless of current edit mode
- Grid glow pass and cell rendering use mode-aware colors (Lead = yellow, Drum = purple)
- Density bar fill color adapts to current mode
- Header badge uses yellow accent for Lead mode
- Version bumped to 0.10.0

### Removed

- `active_mode` field from `MatrixApp` (no longer needed — mode only affects UI, not audio)

## [0.9.0] - 2026-02-15

### Added

- **Drummer mode** for Matrix sequencer: each row is a percussion instrument instead of a pitched note
- 16-instrument drum kit: Crash, Ride, Open HH, Closed HH, Clap, Rimshot, Snare, Tom Hi, Tom Mid, Tom Low, Conga Hi, Conga Lo, Cowbell, Claves, Kick, Kick Hard
- Fixed-topology drum voice engine (`src/engine/drum.rs`): dual-source synthesis (noise + sine) mixed via `Shared` atomics — zero graph rebuilds between hits
- `build_drum_voice_unit()` and `build_drum_poly_graph()` for 8-voice drum polyphony
- Mode toggle: switch between Lead (melodic) and Drummer (percussion) with M key or UI button
- Independent grids per mode: Lead and Drummer grids are preserved when switching
- Drum Kit sidebar panel with global modifiers: Tune (pitch offset), Decay, Color (filter cutoff multiplier)
- Drum accent colors (amber/orange) for Drummer mode header badge
- `ChannelMode` enum (Lead, Drummer) in `MatrixState` with `active_grid()` / `active_grid_mut()` abstraction
- 15 new unit tests: 7 in `engine/drum.rs` (kit entries, voice build, sound production, param changes), 8 in `state.rs` (mode switching, grid independence, label changes)

### Changed

- Grid, transport, header, shortcuts, and sidebar conditionally adapt to active mode
- `MatrixApp` bridge supports dual audio graphs (Lead: `build_poly_graph`, Drummer: `build_drum_poly_graph`)
- Mode switch triggers one-time graph rebuild; effects chain shared between modes
- Undo/redo and draw modes operate on the active mode's grid
- Footer shortcuts updated with M (mode toggle)
- Version bumped to 0.9.0

## [0.8.0] - 2026-02-15

### Added

- **Matrix sequencer mode** (`--matrix`): 16x16 matrix sequencer inspired by Yamaha Matrix
- `MatrixSynth` UI module (`src/matrix_synth/`) with 19 files: state, theme, grid, transport, history, header, density bar, shortcuts, 4 widgets, 7 panels, mod.rs
- `MatrixApp` bridge (`src/gui/matrix_app.rs`): maps UI state to audio engine parameters, triggers notes on playhead advancement
- 16x16 grid matrix: rows = notes (C5 to A3), columns = time steps, with click/drag interaction
- 3 draw modes: Toggle, Draw, Erase (keyboard shortcuts D/E/T)
- Playhead with left-to-right advancement, triggering active cells as notes
- Transport controls: play/pause, BPM slider (40–240), swing knob
- Scale selector: Chromatic, Major, Minor, Pentatonic with MIDI note mapping
- Sidebar synth panels: oscillator (waveform, pitch, detune), ADSR envelope, resonant filter (LP/HP/BP), LFO (pitch/filter/amp modulation), effects (reverb, delay, chorus)
- Custom Matrix widgets: rotary knob (configurable size, drag, double-click reset), horizontal slider, toggle button group, panel wrapper
- Density bar: per-column note density visualization aligned with grid
- Header with LED status indicator, title, version, active note badges
- Undo/redo system: 20-entry circular history (Ctrl+Z / Ctrl+Shift+Z)
- Modifier clicks: Shift+click toggles entire row, Ctrl+click toggles entire column
- Keyboard shortcuts: Space (play/pause), C (clear), Ctrl+Up/Down (BPM), Ctrl+1-4 (waveform), Tab (filter cycle), Shift+Tab (LFO target)
- 8-voice polyphony with voice stealing for sequenced notes
- Playhead column background highlight and cell glow effects (hit, on, hover states)
- 23 new unit tests: 18 in `state.rs` (state operations, enums, scale mapping, MIDI) + 5 in `history.rs` (undo/redo, limits)
- `run_matrix()` entry point in `gui/mod.rs` (1100x750 window)

### Changed

- `main.rs`: added `--matrix` CLI flag alongside existing `--gui`
- Version bumped to 0.8.0

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
