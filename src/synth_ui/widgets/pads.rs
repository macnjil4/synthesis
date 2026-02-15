use eframe::egui::{self, pos2, vec2, Sense, Stroke, Ui};

use crate::midi::NoteEvent;
use crate::synth_ui::theme::SynthTheme;

const PAD_LABELS: [&str; 16] = [
    "Kick", "Snare", "HiHat", "Clap",
    "Tom1", "Tom2", "Rim", "Cow",
    "Crash", "Ride", "Shaker", "Perc1",
    "Perc2", "Perc3", "FX1", "FX2",
];

/// MIDI notes for each pad (GM drum map starting at C2=36).
pub const PAD_NOTES: [u8; 16] = [
    36, 38, 42, 39,  // Kick, Snare, HiHat, Clap
    41, 43, 37, 56,  // Tom1, Tom2, Rim, Cow
    49, 51, 70, 60,  // Crash, Ride, Shaker, Perc1
    61, 62, 63, 64,  // Perc2, Perc3, FX1, FX2
];

/// Minimum frames a pad note stays on (ensures audible ADSR attack).
const MIN_TRIGGER_FRAMES: u32 = 12; // ~200ms at 60fps

pub fn draw(ui: &mut Ui) -> Vec<NoteEvent> {
    let mut events = Vec::new();
    let available = ui.available_width().min(ui.available_height());
    let pad_size = (available - 3.0 * 4.0) / 4.0; // 4 columns, 3 gaps

    egui::Grid::new("drum_pads")
        .spacing(vec2(4.0, 4.0))
        .show(ui, |ui| {
            for row in 0..4 {
                for col in 0..4 {
                    let idx = row * 4 + col;
                    let label = PAD_LABELS[idx];
                    let note = PAD_NOTES[idx];

                    let size = vec2(pad_size, pad_size);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                    let painter = ui.painter_at(rect);

                    // Track trigger countdown per pad
                    let trigger_id = ui.id().with(("pad_trigger", idx));
                    let remaining: u32 = ui.data(|d| d.get_temp(trigger_id)).unwrap_or(0);

                    let is_active = remaining > 0;
                    let just_clicked = response.clicked();
                    let is_hovered = response.hovered();

                    let bg = if is_active || (response.is_pointer_button_down_on()) {
                        SynthTheme::ACCENT_DARK
                    } else if is_hovered {
                        SynthTheme::PANEL_LIGHT
                    } else {
                        SynthTheme::PAD_IDLE
                    };

                    let border = if is_active {
                        SynthTheme::ACCENT_LIGHT
                    } else {
                        SynthTheme::BORDER
                    };

                    painter.rect_filled(rect, 6.0, bg);
                    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);

                    // Label
                    painter.text(
                        pos2(rect.center().x, rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::FontId::proportional(10.0),
                        if is_active { SynthTheme::TEXT } else { SynthTheme::TEXT_DIM },
                    );

                    // F-key hint
                    if idx < 6 {
                        let hint = format!("F{}", idx + 1);
                        painter.text(
                            pos2(rect.right() - 4.0, rect.top() + 4.0),
                            egui::Align2::RIGHT_TOP,
                            hint,
                            egui::FontId::proportional(7.0),
                            SynthTheme::BORDER,
                        );
                    }

                    // Trigger logic: click starts a countdown, NoteOff fires when countdown reaches 0
                    if just_clicked && remaining == 0 {
                        events.push(NoteEvent::On { note, velocity: 120 });
                        ui.data_mut(|d| d.insert_temp(trigger_id, MIN_TRIGGER_FRAMES));
                    } else if remaining > 0 {
                        let new_remaining = remaining - 1;
                        if new_remaining == 0 {
                            events.push(NoteEvent::Off { note });
                        }
                        ui.data_mut(|d| d.insert_temp(trigger_id, new_remaining));
                    }
                }
                ui.end_row();
            }
        });

    events
}
