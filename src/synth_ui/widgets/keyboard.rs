use std::collections::HashSet;

use eframe::egui::{self, pos2, vec2, Stroke};

use crate::midi::NoteEvent;
use crate::synth_ui::theme::SynthTheme;

const NOTES_PER_OCTAVE: u8 = 12;
const NUM_OCTAVES: u8 = 2;
const BASE_NOTE: u8 = 48; // C3
const WHITE_KEY_OFFSETS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
const BLACK_KEY_INFO: [(u8, usize); 5] = [
    (1, 0),  // C#
    (3, 1),  // D#
    (6, 3),  // F#
    (8, 4),  // G#
    (10, 5), // A#
];
const NOTE_NAMES: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];

pub fn draw(ui: &mut egui::Ui, pressed_keys: &HashSet<u8>) -> Vec<NoteEvent> {
    let mut events = Vec::new();

    let total_white_keys = NUM_OCTAVES as usize * 7;
    let available_width = ui.available_width();
    let white_key_width = (available_width / total_white_keys as f32).min(38.0);
    let white_key_height = 120.0;
    let black_key_width = white_key_width * 0.6;
    let black_key_height = white_key_height * 0.6;

    let keyboard_width = white_key_width * total_white_keys as f32;

    let (response, painter) = ui.allocate_painter(
        vec2(keyboard_width, white_key_height),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let origin = rect.left_top();

    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let pointer_pressed = ui.input(|i| i.pointer.primary_down());
    let pointer_released = ui.input(|i| i.pointer.primary_released());

    let mut mouse_note: Option<u8> = None;

    // White keys
    for octave in 0..NUM_OCTAVES {
        for (key_idx, &offset) in WHITE_KEY_OFFSETS.iter().enumerate() {
            let midi_note = BASE_NOTE + octave * NOTES_PER_OCTAVE + offset;
            let white_idx = octave as usize * 7 + key_idx;
            let x = origin.x + white_idx as f32 * white_key_width;

            let key_rect = egui::Rect::from_min_size(
                pos2(x, origin.y),
                vec2(white_key_width, white_key_height),
            );

            let is_hovered = pointer_pos.is_some_and(|p| key_rect.contains(p));
            let is_pressed = pressed_keys.contains(&midi_note);

            let fill = if is_pressed || (is_hovered && pointer_pressed) {
                SynthTheme::ACCENT_LIGHT
            } else if is_hovered {
                egui::Color32::from_rgb(210, 200, 220)
            } else {
                SynthTheme::WHITE_KEY
            };

            // Rounded bottom corners only
            let rounding = egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: 6,
                se: 6,
            };
            painter.rect_filled(key_rect, rounding, fill);
            painter.rect_stroke(
                key_rect,
                rounding,
                Stroke::new(1.0, SynthTheme::BORDER),
                egui::StrokeKind::Inside,
            );

            // Note name at bottom
            let octave_num = 3 + octave;
            let name = format!("{}{}", NOTE_NAMES[key_idx], octave_num);
            painter.text(
                pos2(x + white_key_width / 2.0, origin.y + white_key_height - 10.0),
                egui::Align2::CENTER_BOTTOM,
                name,
                egui::FontId::proportional(9.0),
                SynthTheme::TEXT_DIM,
            );

            if is_hovered && pointer_pressed {
                mouse_note = Some(midi_note);
            }
        }
    }

    // Black keys
    for octave in 0..NUM_OCTAVES {
        for &(offset, after_white) in &BLACK_KEY_INFO {
            let midi_note = BASE_NOTE + octave * NOTES_PER_OCTAVE + offset;
            let white_idx = octave as usize * 7 + after_white;
            let x = origin.x + (white_idx as f32 + 1.0) * white_key_width - black_key_width / 2.0;

            let key_rect = egui::Rect::from_min_size(
                pos2(x, origin.y),
                vec2(black_key_width, black_key_height),
            );

            let is_hovered = pointer_pos.is_some_and(|p| key_rect.contains(p));
            let is_pressed = pressed_keys.contains(&midi_note);

            let fill = if is_pressed || (is_hovered && pointer_pressed) {
                SynthTheme::ACCENT
            } else if is_hovered {
                SynthTheme::PANEL_LIGHT
            } else {
                SynthTheme::BLACK_KEY
            };

            let rounding = egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            };
            painter.rect_filled(key_rect, rounding, fill);
            painter.rect_stroke(
                key_rect,
                rounding,
                Stroke::new(1.0, egui::Color32::BLACK),
                egui::StrokeKind::Inside,
            );

            if is_hovered && pointer_pressed {
                mouse_note = Some(midi_note);
            }
        }
    }

    // Generate mouse note events
    let id = ui.id().with("synth_keyboard_state");
    let prev_note: Option<u8> = ui.data(|d| d.get_temp(id)).unwrap_or(None);

    if pointer_pressed && rect.contains(pointer_pos.unwrap_or_default()) {
        if mouse_note != prev_note {
            if let Some(prev) = prev_note {
                events.push(NoteEvent::Off { note: prev });
            }
            if let Some(note) = mouse_note {
                events.push(NoteEvent::On { note, velocity: 100 });
            }
        }
        ui.data_mut(|d| d.insert_temp(id, mouse_note));
    } else if pointer_released || (!pointer_pressed && prev_note.is_some()) {
        if let Some(prev) = prev_note {
            events.push(NoteEvent::Off { note: prev });
        }
        ui.data_mut(|d| d.insert_temp::<Option<u8>>(id, None));
    }

    events
}
