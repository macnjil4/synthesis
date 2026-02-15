use eframe::egui;
use eframe::egui::Color32;

use super::state::{ChannelMode, TenoriState};
use super::theme::Theme;

pub fn draw(ui: &mut egui::Ui, state: &TenoriState) {
    ui.horizontal(|ui| {
        // LED
        let (led_rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        let led_color = if state.is_playing {
            Theme::LED_PLAYING
        } else {
            Theme::LED_STOPPED
        };
        let glow_color = if state.is_playing {
            Theme::LED_PLAYING_GLOW
        } else {
            Theme::ACCENT_LIGHT
        };

        // Glow behind
        ui.painter().circle_filled(
            led_rect.center(),
            8.0,
            Color32::from_rgba_premultiplied(glow_color.r(), glow_color.g(), glow_color.b(), 40),
        );
        ui.painter()
            .circle_filled(led_rect.center(), 5.0, led_color);

        ui.add_space(8.0);

        // Title
        ui.label(
            egui::RichText::new("TENORI-SYNTH")
                .size(20.0)
                .strong()
                .color(Theme::ACCENT_LIGHT),
        );

        ui.add_space(8.0);

        // Mode badge
        let (mode_label, mode_color) = match state.mode {
            ChannelMode::Lead => ("LEAD", Theme::LEAD_ACCENT_DARK),
            ChannelMode::Drummer => ("DRUMS", Theme::DRUM_ACCENT_DARK),
            ChannelMode::Bass => ("BASS", Theme::BASS_ACCENT_DARK),
        };
        egui::Frame::new()
            .fill(mode_color)
            .corner_radius(3.0)
            .inner_margin(egui::Margin::symmetric(6, 2))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(mode_label)
                        .size(9.0)
                        .strong()
                        .color(Theme::TEXT_WHITE),
                );
            });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Version
            ui.label(egui::RichText::new("v0.12.0").size(9.0).color(Theme::TEXT_DIM));
            ui.add_space(12.0);

            // Active notes (badges)
            let notes = state.active_note_names();
            for note in notes.iter().rev() {
                egui::Frame::new()
                    .fill(Theme::ACCENT_DARK)
                    .corner_radius(3.0)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(*note)
                                .size(10.0)
                                .strong()
                                .color(Theme::TEXT_WHITE),
                        );
                    });
            }

            if !notes.is_empty() {
                ui.label(
                    egui::RichText::new("PLAYING:")
                        .size(9.0)
                        .color(Theme::TEXT_DIM),
                );
            }
        });
    });
}
