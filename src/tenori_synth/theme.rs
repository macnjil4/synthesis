use eframe::egui::{Color32, CornerRadius, Shadow, Stroke};

pub struct Theme;

impl Theme {
    // ── Backgrounds ──
    pub const BG: Color32 = Color32::from_rgb(13, 13, 26);
    pub const BG_GRADIENT_TOP: Color32 = Color32::from_rgba_premultiplied(124, 58, 237, 20);
    pub const PANEL: Color32 = Color32::from_rgb(26, 26, 46);
    pub const PANEL_LIGHT: Color32 = Color32::from_rgb(34, 34, 58);
    pub const BORDER: Color32 = Color32::from_rgb(42, 42, 74);

    // ── Accents ──
    pub const ACCENT: Color32 = Color32::from_rgb(155, 89, 182);
    pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(192, 132, 252);
    pub const ACCENT_DARK: Color32 = Color32::from_rgb(124, 58, 237);

    // ── Text ──
    pub const TEXT: Color32 = Color32::from_rgb(224, 216, 240);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(136, 120, 169);
    pub const TEXT_WHITE: Color32 = Color32::WHITE;

    // ── Grid ──
    pub const CELL_OFF: Color32 = Color32::from_rgb(20, 20, 40);
    pub const CELL_OFF_HOVER: Color32 = Color32::from_rgb(28, 28, 52);
    pub const CELL_ON: Color32 = Color32::from_rgb(124, 58, 237);
    pub const CELL_ON_HOVER: Color32 = Color32::from_rgb(140, 75, 245);
    pub const CELL_PLAYHEAD: Color32 = Color32::from_rgb(26, 26, 56);
    pub const CELL_PLAYHEAD_BORDER: Color32 = Color32::from_rgb(58, 58, 90);
    pub const CELL_HIT: Color32 = Color32::from_rgb(233, 213, 255);
    pub const CELL_HIT_GLOW: Color32 = Color32::from_rgba_premultiplied(192, 132, 252, 40);
    pub const PLAYHEAD_BG: Color32 = Color32::from_rgba_premultiplied(192, 132, 252, 38);

    // ── Widgets ──
    pub const KNOB_BG: Color32 = Color32::from_rgb(22, 22, 43);
    pub const LED_PLAYING: Color32 = Color32::from_rgb(74, 222, 128);
    pub const LED_PLAYING_GLOW: Color32 = Color32::from_rgba_premultiplied(74, 222, 128, 128);
    pub const LED_STOPPED: Color32 = Color32::from_rgb(192, 132, 252);

    // ── Lead mode accents (yellow/gold) ──
    #[allow(dead_code)]
    pub const LEAD_ACCENT: Color32 = Color32::from_rgb(234, 179, 8);
    #[allow(dead_code)]
    pub const LEAD_ACCENT_LIGHT: Color32 = Color32::from_rgb(250, 204, 21);
    pub const LEAD_ACCENT_DARK: Color32 = Color32::from_rgb(202, 138, 4);
    pub const LEAD_CELL_ON: Color32 = Color32::from_rgb(202, 138, 4);
    pub const LEAD_CELL_ON_HOVER: Color32 = Color32::from_rgb(234, 179, 8);
    pub const LEAD_CELL_HIT: Color32 = Color32::from_rgb(254, 240, 138);
    pub const LEAD_CELL_HIT_GLOW: Color32 = Color32::from_rgba_premultiplied(250, 204, 21, 40);

    // ── Drum mode accents (amber/orange) ──
    #[allow(dead_code)]
    pub const DRUM_ACCENT: Color32 = Color32::from_rgb(217, 119, 6);
    #[allow(dead_code)]
    pub const DRUM_ACCENT_LIGHT: Color32 = Color32::from_rgb(251, 191, 36);
    pub const DRUM_ACCENT_DARK: Color32 = Color32::from_rgb(180, 83, 9);

    // ── Bass mode accents (pink/rose) ──
    #[allow(dead_code)]
    pub const BASS_ACCENT: Color32 = Color32::from_rgb(236, 72, 153);
    #[allow(dead_code)]
    pub const BASS_ACCENT_LIGHT: Color32 = Color32::from_rgb(244, 114, 182);
    pub const BASS_ACCENT_DARK: Color32 = Color32::from_rgb(219, 39, 119);
    pub const BASS_CELL_ON: Color32 = Color32::from_rgb(219, 39, 119);
    pub const BASS_CELL_ON_HOVER: Color32 = Color32::from_rgb(236, 72, 153);
    pub const BASS_CELL_HIT: Color32 = Color32::from_rgb(251, 207, 232);
    pub const BASS_CELL_HIT_GLOW: Color32 = Color32::from_rgba_premultiplied(244, 114, 182, 40);

    // ── Ghost overlay (other mode's cells visible in transparency) ──
    pub const GHOST_LEAD: Color32 = Color32::from_rgba_premultiplied(234, 179, 8, 30);
    pub const GHOST_DRUM: Color32 = Color32::from_rgba_premultiplied(124, 58, 237, 30);
    pub const GHOST_BASS: Color32 = Color32::from_rgba_premultiplied(236, 72, 153, 30);

    // ── Dimensions ──
    pub const CELL_SIZE: f32 = 32.0;
    pub const CELL_GAP: f32 = 2.0;
    pub const CELL_ROUNDING: f32 = 4.0;
    pub const PANEL_ROUNDING: u8 = 10;
    pub const SIDEBAR_WIDTH: f32 = 210.0;
    pub const NOTE_LABEL_WIDTH: f32 = 38.0;
    pub const COL_HEADER_HEIGHT: f32 = 16.0;
    pub const DENSITY_HEIGHT: f32 = 10.0;

    // ── Row controls (right of grid) ──
    pub const ROW_CTRL_GAP: f32 = 6.0;
    pub const MUTE_BTN_SIZE: f32 = 20.0;
    pub const VOL_BAR_WIDTH: f32 = 48.0;
    pub const VOL_BAR_HEIGHT: f32 = 12.0;
    pub const CTRL_GAP: f32 = 4.0;

    pub const MUTE_ON: Color32 = Color32::from_rgb(220, 38, 38);
    pub const MUTE_OFF: Color32 = Color32::from_rgb(42, 42, 74);
    pub const VOL_BAR_BG: Color32 = Color32::from_rgb(20, 20, 40);
    pub const VOL_BAR_FILL: Color32 = Color32::from_rgb(100, 100, 160);
    pub const MUTED_ROW_OVERLAY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 100);

    // ── Shadows ──
    pub fn panel_shadow() -> Shadow {
        Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 128),
        }
    }

    pub fn panel_stroke() -> Stroke {
        Stroke::new(1.0, Self::BORDER)
    }

    pub fn panel_rounding() -> CornerRadius {
        CornerRadius::same(Self::PANEL_ROUNDING)
    }
}
