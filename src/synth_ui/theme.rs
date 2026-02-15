use eframe::egui::{self, Color32, Frame, Margin, Shadow, Stroke};

pub struct SynthTheme;

impl SynthTheme {
    pub const BG: Color32 = Color32::from_rgb(13, 13, 26);
    pub const PANEL: Color32 = Color32::from_rgb(26, 26, 46);
    pub const PANEL_LIGHT: Color32 = Color32::from_rgb(34, 34, 58);
    pub const BORDER: Color32 = Color32::from_rgb(42, 42, 74);
    pub const ACCENT: Color32 = Color32::from_rgb(155, 89, 182);
    pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(192, 132, 252);
    pub const ACCENT_DARK: Color32 = Color32::from_rgb(124, 58, 237);
    pub const TEXT: Color32 = Color32::from_rgb(224, 216, 240);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(136, 120, 169);
    pub const KNOB_BG: Color32 = Color32::from_rgb(22, 22, 43);
    pub const PAD_IDLE: Color32 = Color32::from_rgb(30, 30, 56);
    pub const WHITE_KEY: Color32 = Color32::from_rgb(232, 224, 240);
    pub const BLACK_KEY: Color32 = Color32::from_rgb(26, 26, 46);
    pub const PANEL_ROUNDING: f32 = 10.0;

    // VU meter colors
    pub const VU_GREEN: Color32 = Color32::from_rgb(100, 220, 100);
    pub const VU_YELLOW: Color32 = Color32::from_rgb(220, 220, 60);
    pub const VU_RED: Color32 = Color32::from_rgb(220, 60, 60);

    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        let visuals = &mut style.visuals;

        visuals.dark_mode = true;
        visuals.panel_fill = Self::BG;
        visuals.window_fill = Self::BG;
        visuals.extreme_bg_color = Self::KNOB_BG;
        visuals.faint_bg_color = Self::PANEL;

        visuals.override_text_color = Some(Self::TEXT);
        visuals.selection.bg_fill = Self::ACCENT_DARK;
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT_LIGHT);

        visuals.widgets.inactive.bg_fill = Self::PANEL;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_DIM);

        visuals.widgets.hovered.bg_fill = Self::PANEL_LIGHT;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::ACCENT);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT);

        visuals.widgets.active.bg_fill = Self::ACCENT_DARK;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::ACCENT_LIGHT);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT);

        visuals.widgets.noninteractive.bg_fill = Self::PANEL;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Self::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_DIM);

        ctx.set_style(style);
    }

    pub fn panel_frame() -> Frame {
        Frame::new()
            .fill(Self::PANEL)
            .stroke(Stroke::new(1.0, Self::BORDER))
            .corner_radius(Self::PANEL_ROUNDING)
            .inner_margin(Margin::same(8))
            .shadow(Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: Color32::from_black_alpha(80),
            })
    }
}
