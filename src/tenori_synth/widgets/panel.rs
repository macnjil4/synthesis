use eframe::egui;

use crate::tenori_synth::theme::Theme;

pub fn synth_panel(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Theme::PANEL)
        .stroke(Theme::panel_stroke())
        .corner_radius(Theme::panel_rounding())
        .shadow(Theme::panel_shadow())
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(10.0)
                    .strong()
                    .color(Theme::ACCENT_LIGHT),
            );
            ui.add_space(10.0);
            add_contents(ui);
        });
    ui.add_space(10.0);
}

pub fn synth_panel_no_title(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Theme::PANEL)
        .stroke(Theme::panel_stroke())
        .corner_radius(Theme::panel_rounding())
        .shadow(Theme::panel_shadow())
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            add_contents(ui);
        });
}
