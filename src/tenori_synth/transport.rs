use eframe::egui;
use eframe::egui::Stroke;

use super::history::History;
use super::state::{ChannelMode, TenoriState};
use super::theme::Theme;
use super::widgets;

pub fn draw(ui: &mut egui::Ui, state: &mut TenoriState, history: &mut History) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // Play/Pause button
        let (btn_rect, btn_response) =
            ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::click());
        let painter = ui.painter_at(btn_rect);
        let center = btn_rect.center();

        if state.is_playing {
            // Playing state: accent bg + pause icon
            painter.circle(center, 20.0, Theme::ACCENT_DARK, Stroke::new(2.0, Theme::ACCENT_LIGHT));
            // Pause icon: two bars
            let bar_w = 4.0;
            let bar_h = 14.0;
            let gap = 3.0;
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(center.x - gap, center.y),
                    egui::vec2(bar_w, bar_h),
                ),
                1.0,
                Theme::TEXT_WHITE,
            );
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(center.x + gap, center.y),
                    egui::vec2(bar_w, bar_h),
                ),
                1.0,
                Theme::TEXT_WHITE,
            );
        } else {
            // Stopped state: knob bg + play triangle
            painter.circle(center, 20.0, Theme::KNOB_BG, Stroke::new(2.0, Theme::BORDER));
            // Play icon: triangle
            let tri_size = 10.0;
            let points = vec![
                egui::pos2(center.x - tri_size * 0.4, center.y - tri_size * 0.6),
                egui::pos2(center.x - tri_size * 0.4, center.y + tri_size * 0.6),
                egui::pos2(center.x + tri_size * 0.6, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points,
                Theme::TEXT_WHITE,
                Stroke::NONE,
            ));
        }

        if btn_response.clicked() {
            state.toggle_play();
        }

        // Clear button
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("CLEAR")
                        .size(10.0)
                        .strong()
                        .color(Theme::TEXT_DIM),
                )
                .fill(Theme::KNOB_BG)
                .stroke(Theme::panel_stroke())
                .corner_radius(6.0),
            )
            .clicked()
        {
            let before = *state.active_grid();
            state.clear_grid();
            history.push(before);
        }

        // Separator
        let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 28.0), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, 0.0, Theme::BORDER);

        // BPM
        ui.label(
            egui::RichText::new("BPM")
                .size(10.0)
                .strong()
                .color(Theme::TEXT),
        );
        ui.add(egui::Slider::new(&mut state.bpm, 40.0..=240.0).show_value(false));
        ui.label(
            egui::RichText::new(format!("{}", state.bpm as i32))
                .size(12.0)
                .strong()
                .color(Theme::ACCENT_LIGHT),
        );

        // Separator
        let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 28.0), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, 0.0, Theme::BORDER);

        // Swing knob
        widgets::knob::knob(ui, "Swing", &mut state.swing, 0.0, 100.0, 0.0, "%", 36.0);

        // Separator
        let (sep_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 28.0), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, 0.0, Theme::BORDER);

        // Mode toggle (Lead / Drums)
        let mode_options: Vec<(ChannelMode, &str)> =
            ChannelMode::ALL.iter().map(|m| (*m, m.label())).collect();
        widgets::select_buttons::select_buttons(ui, &mut state.mode, &mode_options);
    });
}
