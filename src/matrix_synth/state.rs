pub const ROWS: usize = 16;
pub const COLS: usize = 16;

/// Notes displayed top (high) to bottom (low)
pub const NOTE_LABELS: [&str; ROWS] = [
    "C5", "B4", "A#4", "A4", "G#4", "G4", "F#4", "F4",
    "E4", "D#4", "D4", "C#4", "C4", "B3", "A#3", "A3",
];

/// Semitone intervals per scale (from lowest note)
pub const SCALE_CHROMATIC: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
pub const SCALE_MAJOR: [u8; 16] = [0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21, 23, 24, 26];
pub const SCALE_MINOR: [u8; 16] = [0, 2, 3, 5, 7, 8, 10, 12, 14, 15, 17, 19, 20, 22, 24, 26];
pub const SCALE_PENTATONIC: [u8; 16] = [
    0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28, 31, 33, 36,
];

/// Base MIDI note (A3 = bottom row in chromatic)
pub const BASE_MIDI_NOTE: u8 = 57;

/// Bass mode: base MIDI note (A1 = bottom row in chromatic)
pub const BASS_BASE_MIDI_NOTE: u8 = 33;

/// Bass note labels (top = high, bottom = low) — A1 to C3
pub const BASS_NOTE_LABELS: [&str; ROWS] = [
    "C3", "B2", "A#2", "A2", "G#2", "G2", "F#2", "F2",
    "E2", "D#2", "D2", "C#2", "C2", "B1", "A#1", "A1",
];

/// Drum instrument labels (top row = index 0, bottom row = index 15)
pub const DRUM_LABELS: [&str; ROWS] = [
    "Crash", "Ride", "O-HH", "C-HH", "Clap", "Rim", "Snare", "TomH",
    "TomM", "TomL", "CngH", "CngL", "Cowbl", "Clave", "Kick", "KickH",
];

// ── Enums ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
    Lead,
    Drummer,
    Bass,
}

impl ChannelMode {
    pub const ALL: [ChannelMode; 3] = [Self::Lead, Self::Drummer, Self::Bass];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Lead => "Lead",
            Self::Drummer => "Drums",
            Self::Bass => "Bass",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum BassPreset {
    SubBass,
    AcidBass,
    FunkBass,
    WarmBass,
    PluckBass,
    GrowlBass,
}

impl BassPreset {
    pub const ALL: [BassPreset; 6] = [
        Self::SubBass,
        Self::AcidBass,
        Self::FunkBass,
        Self::WarmBass,
        Self::PluckBass,
        Self::GrowlBass,
    ];
    pub fn label(&self) -> &'static str {
        match self {
            Self::SubBass => "Sub Bass",
            Self::AcidBass => "Acid Bass",
            Self::FunkBass => "Funk Bass",
            Self::WarmBass => "Warm Bass",
            Self::PluckBass => "Pluck Bass",
            Self::GrowlBass => "Growl Bass",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumPreset {
    LinnDrum,
    TR505,
    CR78,
}

impl DrumPreset {
    pub const ALL: [DrumPreset; 3] = [Self::LinnDrum, Self::TR505, Self::CR78];
    pub fn label(&self) -> &'static str {
        match self {
            Self::LinnDrum => "LinnDrum",
            Self::TR505 => "TR-505",
            Self::CR78 => "CR-78",
        }
    }
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::LinnDrum => "lm2",
            Self::TR505 => "tr505",
            Self::CR78 => "cr78",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Tri,
}

impl Waveform {
    pub const ALL: [Waveform; 4] = [Self::Sine, Self::Saw, Self::Square, Self::Tri];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Saw => "Saw",
            Self::Square => "Square",
            Self::Tri => "Tri",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LP,
    HP,
    BP,
}

impl FilterType {
    pub const ALL: [FilterType; 3] = [Self::LP, Self::HP, Self::BP];
    pub fn label(&self) -> &'static str {
        match self {
            Self::LP => "LP",
            Self::HP => "HP",
            Self::BP => "BP",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            Self::LP => Self::HP,
            Self::HP => Self::BP,
            Self::BP => Self::LP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoDest {
    Pitch,
    Filter,
    Amp,
}

impl LfoDest {
    pub const ALL: [LfoDest; 3] = [Self::Pitch, Self::Filter, Self::Amp];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pitch => "Pitch",
            Self::Filter => "Filter",
            Self::Amp => "Amp",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            Self::Pitch => Self::Filter,
            Self::Filter => Self::Amp,
            Self::Amp => Self::Pitch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    Toggle,
    Draw,
    Erase,
}

impl DrawMode {
    #[allow(dead_code)]
    pub const ALL: [DrawMode; 3] = [Self::Toggle, Self::Draw, Self::Erase];
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Toggle => "Toggle",
            Self::Draw => "Draw",
            Self::Erase => "Erase",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    Pentatonic,
}

impl Scale {
    pub const ALL: [Scale; 4] = [Self::Chromatic, Self::Major, Self::Minor, Self::Pentatonic];
    pub fn label(&self) -> &'static str {
        match self {
            Self::Chromatic => "Chromatic",
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Pentatonic => "Pentatonic",
        }
    }
    pub fn intervals(&self) -> &'static [u8; 16] {
        match self {
            Self::Chromatic => &SCALE_CHROMATIC,
            Self::Major => &SCALE_MAJOR,
            Self::Minor => &SCALE_MINOR,
            Self::Pentatonic => &SCALE_PENTATONIC,
        }
    }
}

// ── Main state ──

#[derive(Clone)]
pub struct MatrixState {
    // Grid matrices (one per mode, preserved when switching)
    pub grid: [[bool; COLS]; ROWS],
    pub drum_grid: [[bool; COLS]; ROWS],
    pub bass_grid: [[bool; COLS]; ROWS],

    // Channel mode
    pub mode: ChannelMode,

    // Transport
    pub is_playing: bool,
    pub play_col: i32,     // -1 = stopped, 0..15 = active column
    pub bpm: f32,          // 40.0 ..= 240.0
    pub swing: f32,        // 0.0 ..= 100.0
    pub elapsed_secs: f64, // accumulator for step advancement

    // Draw mode
    pub draw_mode: DrawMode,

    // Scale
    pub scale: Scale,

    // Oscillator
    pub osc_waveform: Waveform,
    pub osc_pitch: f32,  // 0.0 ..= 100.0
    pub osc_detune: f32, // 0.0 ..= 100.0 (cents)

    // ADSR
    pub env_attack: f32,  // 0.0 ..= 100.0
    pub env_decay: f32,
    pub env_sustain: f32,
    pub env_release: f32,

    // Filter
    pub filter_type: FilterType,
    pub filter_cutoff: f32, // 0.0 ..= 100.0
    pub filter_reso: f32,

    // LFO
    pub lfo_rate: f32,
    pub lfo_depth: f32,
    pub lfo_dest: LfoDest,

    // Effects
    pub fx_reverb: f32,
    pub fx_delay: f32,
    pub fx_chorus: f32,

    // Drum kit
    pub drum_preset: DrumPreset,
    pub drum_tune: f32,  // 0.0 ..= 100.0
    pub drum_decay: f32,
    pub drum_color: f32,

    // Bass preset
    pub bass_preset: BassPreset,

    // Per-row mute & volume (independent per mode)
    pub lead_row_mute:   [bool; ROWS],
    pub lead_row_volume: [f32; ROWS],
    pub drum_row_mute:   [bool; ROWS],
    pub drum_row_volume: [f32; ROWS],
    pub bass_row_mute:   [bool; ROWS],
    pub bass_row_volume: [f32; ROWS],
}

impl Default for MatrixState {
    fn default() -> Self {
        Self {
            grid: [[false; COLS]; ROWS],
            drum_grid: [[false; COLS]; ROWS],
            bass_grid: [[false; COLS]; ROWS],
            mode: ChannelMode::Lead,
            is_playing: false,
            play_col: -1,
            bpm: 120.0,
            swing: 0.0,
            elapsed_secs: 0.0,
            draw_mode: DrawMode::Toggle,
            scale: Scale::Chromatic,
            osc_waveform: Waveform::Saw,
            osc_pitch: 50.0,
            osc_detune: 0.0,
            env_attack: 10.0,
            env_decay: 30.0,
            env_sustain: 70.0,
            env_release: 40.0,
            filter_type: FilterType::LP,
            filter_cutoff: 75.0,
            filter_reso: 30.0,
            lfo_rate: 30.0,
            lfo_depth: 50.0,
            lfo_dest: LfoDest::Filter,
            fx_reverb: 35.0,
            fx_delay: 20.0,
            fx_chorus: 15.0,
            drum_preset: DrumPreset::LinnDrum,
            drum_tune: 50.0,
            drum_decay: 50.0,
            drum_color: 50.0,
            bass_preset: BassPreset::SubBass,
            lead_row_mute: [false; ROWS],
            lead_row_volume: [1.0; ROWS],
            drum_row_mute: [false; ROWS],
            drum_row_volume: [1.0; ROWS],
            bass_row_mute: [false; ROWS],
            bass_row_volume: [1.0; ROWS],
        }
    }
}

impl MatrixState {
    /// Return the active grid based on current mode.
    pub fn active_grid(&self) -> &[[bool; COLS]; ROWS] {
        match self.mode {
            ChannelMode::Lead => &self.grid,
            ChannelMode::Drummer => &self.drum_grid,
            ChannelMode::Bass => &self.bass_grid,
        }
    }

    /// Return mutable reference to the active grid.
    pub fn active_grid_mut(&mut self) -> &mut [[bool; COLS]; ROWS] {
        match self.mode {
            ChannelMode::Lead => &mut self.grid,
            ChannelMode::Drummer => &mut self.drum_grid,
            ChannelMode::Bass => &mut self.bass_grid,
        }
    }

    /// Row labels for the current mode.
    pub fn row_labels(&self) -> &[&'static str; ROWS] {
        match self.mode {
            ChannelMode::Lead => &NOTE_LABELS,
            ChannelMode::Drummer => &DRUM_LABELS,
            ChannelMode::Bass => &BASS_NOTE_LABELS,
        }
    }

    pub fn clear_grid(&mut self) {
        *self.active_grid_mut() = [[false; COLS]; ROWS];
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            self.play_col = 0;
            self.elapsed_secs = 0.0;
        } else {
            self.play_col = -1;
        }
    }

    pub fn toggle_row(&mut self, row: usize) {
        let grid = self.active_grid_mut();
        let all_on = grid[row].iter().all(|&v| v);
        for col in 0..COLS {
            grid[row][col] = !all_on;
        }
    }

    pub fn toggle_col(&mut self, col: usize) {
        let grid = self.active_grid_mut();
        let all_on = (0..ROWS).all(|r| grid[r][col]);
        for row in grid.iter_mut() {
            row[col] = !all_on;
        }
    }

    /// Active row indices at current column
    pub fn active_rows(&self) -> Vec<usize> {
        if self.play_col < 0 {
            return vec![];
        }
        let col = self.play_col as usize;
        let grid = self.active_grid();
        (0..ROWS).filter(|&r| grid[r][col]).collect()
    }

    /// Active names at current column (notes or drum labels)
    pub fn active_note_names(&self) -> Vec<&'static str> {
        let labels = self.row_labels();
        self.active_rows()
            .iter()
            .map(|&r| labels[r])
            .collect()
    }

    /// Number of active cells in a column
    pub fn col_density(&self, col: usize) -> usize {
        let grid = self.active_grid();
        (0..ROWS).filter(|&r| grid[r][col]).count()
    }

    /// Get MIDI note for a row index using current scale (Lead mode only)
    pub fn row_to_midi(&self, row: usize) -> u8 {
        let intervals = self.scale.intervals();
        // Row 15 = lowest note = base + intervals[0]
        // Row 0 = highest note = base + intervals[15]
        let idx = ROWS - 1 - row;
        BASE_MIDI_NOTE + intervals[idx]
    }

    /// Get MIDI note for a row index using current scale (Bass mode)
    pub fn row_to_bass_midi(&self, row: usize) -> u8 {
        let intervals = self.scale.intervals();
        let idx = ROWS - 1 - row;
        BASS_BASE_MIDI_NOTE + intervals[idx]
    }

    pub fn active_row_mute(&self) -> &[bool; ROWS] {
        match self.mode {
            ChannelMode::Lead => &self.lead_row_mute,
            ChannelMode::Drummer => &self.drum_row_mute,
            ChannelMode::Bass => &self.bass_row_mute,
        }
    }

    pub fn active_row_mute_mut(&mut self) -> &mut [bool; ROWS] {
        match self.mode {
            ChannelMode::Lead => &mut self.lead_row_mute,
            ChannelMode::Drummer => &mut self.drum_row_mute,
            ChannelMode::Bass => &mut self.bass_row_mute,
        }
    }

    pub fn active_row_volume(&self) -> &[f32; ROWS] {
        match self.mode {
            ChannelMode::Lead => &self.lead_row_volume,
            ChannelMode::Drummer => &self.drum_row_volume,
            ChannelMode::Bass => &self.bass_row_volume,
        }
    }

    pub fn active_row_volume_mut(&mut self) -> &mut [f32; ROWS] {
        match self.mode {
            ChannelMode::Lead => &mut self.lead_row_volume,
            ChannelMode::Drummer => &mut self.drum_row_volume,
            ChannelMode::Bass => &mut self.bass_row_volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let s = MatrixState::default();
        assert!(!s.is_playing);
        assert_eq!(s.play_col, -1);
        assert_eq!(s.bpm, 120.0);
        assert_eq!(s.scale, Scale::Chromatic);
        assert_eq!(s.osc_waveform, Waveform::Saw);
        assert_eq!(s.draw_mode, DrawMode::Toggle);
    }

    #[test]
    fn grid_starts_empty() {
        let s = MatrixState::default();
        for row in 0..ROWS {
            for col in 0..COLS {
                assert!(!s.grid[row][col]);
            }
        }
    }

    #[test]
    fn toggle_play_starts_and_stops() {
        let mut s = MatrixState::default();
        s.toggle_play();
        assert!(s.is_playing);
        assert_eq!(s.play_col, 0);

        s.toggle_play();
        assert!(!s.is_playing);
        assert_eq!(s.play_col, -1);
    }

    #[test]
    fn clear_grid_clears_all() {
        let mut s = MatrixState::default();
        s.grid[0][0] = true;
        s.grid[5][10] = true;
        s.grid[15][15] = true;
        s.clear_grid();
        for row in 0..ROWS {
            for col in 0..COLS {
                assert!(!s.grid[row][col]);
            }
        }
    }

    #[test]
    fn toggle_row() {
        let mut s = MatrixState::default();
        // Toggle row 3: all off -> all on
        s.toggle_row(3);
        assert!(s.grid[3].iter().all(|&v| v));
        // Toggle again: all on -> all off
        s.toggle_row(3);
        assert!(s.grid[3].iter().all(|&v| !v));
    }

    #[test]
    fn toggle_col() {
        let mut s = MatrixState::default();
        s.toggle_col(5);
        for row in 0..ROWS {
            assert!(s.grid[row][5]);
        }
        s.toggle_col(5);
        for row in 0..ROWS {
            assert!(!s.grid[row][5]);
        }
    }

    #[test]
    fn active_rows_when_stopped() {
        let s = MatrixState::default();
        assert!(s.active_rows().is_empty());
    }

    #[test]
    fn active_rows_when_playing() {
        let mut s = MatrixState::default();
        s.grid[2][0] = true;
        s.grid[5][0] = true;
        s.toggle_play(); // play_col = 0
        let rows = s.active_rows();
        assert_eq!(rows, vec![2, 5]);
    }

    #[test]
    fn active_note_names() {
        let mut s = MatrixState::default();
        s.grid[0][0] = true; // C5
        s.grid[12][0] = true; // C4
        s.toggle_play();
        let names = s.active_note_names();
        assert_eq!(names, vec!["C5", "C4"]);
    }

    #[test]
    fn col_density() {
        let mut s = MatrixState::default();
        assert_eq!(s.col_density(0), 0);
        s.grid[0][0] = true;
        s.grid[3][0] = true;
        s.grid[7][0] = true;
        assert_eq!(s.col_density(0), 3);
    }

    #[test]
    fn row_to_midi_chromatic() {
        let s = MatrixState::default();
        // Row 0 = C5 = MIDI 72
        assert_eq!(s.row_to_midi(0), 72);
        // Row 15 = A3 = MIDI 57
        assert_eq!(s.row_to_midi(15), 57);
        // Row 12 = C4 = MIDI 60
        assert_eq!(s.row_to_midi(12), 60);
    }

    #[test]
    fn row_to_midi_major_scale() {
        let mut s = MatrixState::default();
        s.scale = Scale::Major;
        // Bottom row: base + major[0] = 57 + 0 = 57
        assert_eq!(s.row_to_midi(15), 57);
        // Row 14: base + major[1] = 57 + 2 = 59
        assert_eq!(s.row_to_midi(14), 59);
    }

    #[test]
    fn scale_intervals_lengths() {
        for scale in Scale::ALL {
            assert_eq!(scale.intervals().len(), 16);
        }
    }

    #[test]
    fn filter_type_cycle() {
        assert_eq!(FilterType::LP.next(), FilterType::HP);
        assert_eq!(FilterType::HP.next(), FilterType::BP);
        assert_eq!(FilterType::BP.next(), FilterType::LP);
    }

    #[test]
    fn lfo_dest_cycle() {
        assert_eq!(LfoDest::Pitch.next(), LfoDest::Filter);
        assert_eq!(LfoDest::Filter.next(), LfoDest::Amp);
        assert_eq!(LfoDest::Amp.next(), LfoDest::Pitch);
    }

    #[test]
    fn waveform_labels() {
        assert_eq!(Waveform::Sine.label(), "Sine");
        assert_eq!(Waveform::Saw.label(), "Saw");
        assert_eq!(Waveform::Square.label(), "Square");
        assert_eq!(Waveform::Tri.label(), "Tri");
    }

    #[test]
    fn draw_mode_labels() {
        assert_eq!(DrawMode::Toggle.label(), "Toggle");
        assert_eq!(DrawMode::Draw.label(), "Draw");
        assert_eq!(DrawMode::Erase.label(), "Erase");
    }

    #[test]
    fn scale_labels() {
        assert_eq!(Scale::Chromatic.label(), "Chromatic");
        assert_eq!(Scale::Major.label(), "Major");
        assert_eq!(Scale::Minor.label(), "Minor");
        assert_eq!(Scale::Pentatonic.label(), "Pentatonic");
    }

    #[test]
    fn default_mode_is_lead() {
        let s = MatrixState::default();
        assert_eq!(s.mode, ChannelMode::Lead);
    }

    #[test]
    fn active_grid_returns_correct_grid() {
        let mut s = MatrixState::default();
        s.grid[0][0] = true;
        s.drum_grid[1][1] = true;

        assert!(s.active_grid()[0][0]);
        assert!(!s.active_grid()[1][1]);

        s.mode = ChannelMode::Drummer;
        assert!(!s.active_grid()[0][0]);
        assert!(s.active_grid()[1][1]);
    }

    #[test]
    fn clear_grid_clears_only_active_mode() {
        let mut s = MatrixState::default();
        s.grid[0][0] = true;
        s.drum_grid[1][1] = true;

        // Clear lead grid
        s.mode = ChannelMode::Lead;
        s.clear_grid();
        assert!(!s.grid[0][0]);
        assert!(s.drum_grid[1][1]); // drum grid preserved
    }

    #[test]
    fn row_labels_change_with_mode() {
        let mut s = MatrixState::default();
        assert_eq!(s.row_labels()[0], "C5");
        s.mode = ChannelMode::Drummer;
        assert_eq!(s.row_labels()[0], "Crash");
    }

    #[test]
    fn drum_grid_independent_from_lead() {
        let mut s = MatrixState::default();
        s.mode = ChannelMode::Lead;
        s.active_grid_mut()[3][5] = true;
        s.mode = ChannelMode::Drummer;
        s.active_grid_mut()[7][2] = true;

        // Verify independence
        assert!(s.grid[3][5]);
        assert!(!s.grid[7][2]);
        assert!(!s.drum_grid[3][5]);
        assert!(s.drum_grid[7][2]);
    }

    #[test]
    fn active_note_names_in_drum_mode() {
        let mut s = MatrixState::default();
        s.mode = ChannelMode::Drummer;
        s.drum_grid[14][0] = true; // Kick
        s.drum_grid[6][0] = true;  // Snare
        s.toggle_play();
        let names = s.active_note_names();
        assert_eq!(names, vec!["Snare", "Kick"]);
    }

    #[test]
    fn channel_mode_labels() {
        assert_eq!(ChannelMode::Lead.label(), "Lead");
        assert_eq!(ChannelMode::Drummer.label(), "Drums");
    }

    #[test]
    fn drum_labels_length() {
        assert_eq!(DRUM_LABELS.len(), ROWS);
    }

    #[test]
    fn bass_preset_labels() {
        assert_eq!(BassPreset::SubBass.label(), "Sub Bass");
        assert_eq!(BassPreset::AcidBass.label(), "Acid Bass");
        assert_eq!(BassPreset::FunkBass.label(), "Funk Bass");
        assert_eq!(BassPreset::WarmBass.label(), "Warm Bass");
        assert_eq!(BassPreset::PluckBass.label(), "Pluck Bass");
        assert_eq!(BassPreset::GrowlBass.label(), "Growl Bass");
    }

    #[test]
    fn bass_preset_all_count() {
        assert_eq!(BassPreset::ALL.len(), 6);
    }

    #[test]
    fn bass_grid_starts_empty() {
        let s = MatrixState::default();
        for row in 0..ROWS {
            for col in 0..COLS {
                assert!(!s.bass_grid[row][col]);
            }
        }
    }

    #[test]
    fn bass_default_preset() {
        let s = MatrixState::default();
        assert_eq!(s.bass_preset, BassPreset::SubBass);
    }

    #[test]
    fn active_grid_bass_mode() {
        let mut s = MatrixState::default();
        s.bass_grid[3][5] = true;
        s.mode = ChannelMode::Bass;
        assert!(s.active_grid()[3][5]);
        assert!(!s.active_grid()[0][0]);
    }

    #[test]
    fn row_labels_bass_mode() {
        let mut s = MatrixState::default();
        s.mode = ChannelMode::Bass;
        assert_eq!(s.row_labels()[0], "C3");
        assert_eq!(s.row_labels()[15], "A1");
    }

    #[test]
    fn row_to_bass_midi_chromatic() {
        let s = MatrixState::default();
        // Row 15 = A1 = MIDI 33
        assert_eq!(s.row_to_bass_midi(15), 33);
        // Row 0 = C3 = MIDI 48
        assert_eq!(s.row_to_bass_midi(0), 48);
    }

    #[test]
    fn bass_note_labels_length() {
        assert_eq!(BASS_NOTE_LABELS.len(), ROWS);
    }

    #[test]
    fn channel_mode_bass_label() {
        assert_eq!(ChannelMode::Bass.label(), "Bass");
    }

    #[test]
    fn channel_mode_all_includes_bass() {
        assert_eq!(ChannelMode::ALL.len(), 3);
        assert!(ChannelMode::ALL.contains(&ChannelMode::Bass));
    }

    #[test]
    fn drum_preset_labels() {
        assert_eq!(DrumPreset::LinnDrum.label(), "LinnDrum");
        assert_eq!(DrumPreset::TR505.label(), "TR-505");
        assert_eq!(DrumPreset::CR78.label(), "CR-78");
    }

    #[test]
    fn drum_preset_dir_names() {
        assert_eq!(DrumPreset::LinnDrum.dir_name(), "lm2");
        assert_eq!(DrumPreset::TR505.dir_name(), "tr505");
        assert_eq!(DrumPreset::CR78.dir_name(), "cr78");
    }

    #[test]
    fn drum_preset_all_count() {
        assert_eq!(DrumPreset::ALL.len(), 3);
    }

    #[test]
    fn drum_default_preset() {
        let s = MatrixState::default();
        assert_eq!(s.drum_preset, DrumPreset::LinnDrum);
    }
}
