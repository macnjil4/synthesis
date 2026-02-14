use eframe::egui;

use crate::midi::NoteEvent;

/// Key layout: true = white key, false = black key.
/// One octave: C, C#, D, D#, E, F, F#, G, G#, A, A#, B
const NOTES_PER_OCTAVE: u8 = 12;
const NUM_OCTAVES: u8 = 2;
const BASE_NOTE: u8 = 48; // C3

/// White key indices within an octave (C=0, D=2, E=4, F=5, G=7, A=9, B=11)
const WHITE_KEY_OFFSETS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Black key offsets and their position relative to white keys.
/// (semitone offset, position as fraction between white keys)
const BLACK_KEY_INFO: [(u8, usize); 5] = [
    (1, 0),  // C# between C(0) and D(1)
    (3, 1),  // D# between D(1) and E(2)
    (6, 3),  // F# between F(3) and G(4)
    (8, 4),  // G# between G(4) and A(5)
    (10, 5), // A# between A(5) and B(6)
];

/// Draw a 2-octave virtual piano keyboard and return note events.
pub fn draw(ui: &mut egui::Ui, base_octave: u8) -> Vec<NoteEvent> {
    let mut events = Vec::new();

    let total_white_keys = (NUM_OCTAVES as usize) * 7;
    let available_width = ui.available_width();
    let white_key_width = (available_width / total_white_keys as f32).min(40.0);
    let white_key_height = 120.0;
    let black_key_width = white_key_width * 0.6;
    let black_key_height = white_key_height * 0.6;

    let keyboard_width = white_key_width * total_white_keys as f32;

    let (response, painter) = ui.allocate_painter(
        egui::vec2(keyboard_width, white_key_height),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let origin = rect.left_top();

    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let pointer_pressed = ui.input(|i| i.pointer.primary_down());
    let pointer_released = ui.input(|i| i.pointer.primary_released());

    // Track which note (if any) is being pressed by the pointer
    let mut pressed_note: Option<u8> = None;

    // Draw white keys first (back layer)
    for octave in 0..NUM_OCTAVES {
        for (key_idx, &offset) in WHITE_KEY_OFFSETS.iter().enumerate() {
            let midi_note = BASE_NOTE + (base_octave - 3) * NOTES_PER_OCTAVE + octave * NOTES_PER_OCTAVE + offset;
            let white_idx = octave as usize * 7 + key_idx;
            let x = origin.x + white_idx as f32 * white_key_width;

            let key_rect = egui::Rect::from_min_size(
                egui::pos2(x, origin.y),
                egui::vec2(white_key_width, white_key_height),
            );

            // Check hover/press (only if not on a black key — checked later)
            let is_hovered = pointer_pos
                .map(|p| key_rect.contains(p))
                .unwrap_or(false);

            let fill = if is_hovered && pointer_pressed {
                egui::Color32::from_rgb(180, 200, 255)
            } else {
                egui::Color32::WHITE
            };

            painter.rect_filled(key_rect, 2.0, fill);
            painter.rect_stroke(key_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::DARK_GRAY), egui::StrokeKind::Inside);

            if is_hovered && pointer_pressed {
                pressed_note = Some(midi_note);
            }
        }
    }

    // Draw black keys (front layer — overwrites press detection)
    for octave in 0..NUM_OCTAVES {
        for &(offset, after_white) in &BLACK_KEY_INFO {
            let midi_note = BASE_NOTE + (base_octave - 3) * NOTES_PER_OCTAVE + octave * NOTES_PER_OCTAVE + offset;
            let white_idx = octave as usize * 7 + after_white;
            let x = origin.x + (white_idx as f32 + 1.0) * white_key_width - black_key_width / 2.0;

            let key_rect = egui::Rect::from_min_size(
                egui::pos2(x, origin.y),
                egui::vec2(black_key_width, black_key_height),
            );

            let is_hovered = pointer_pos
                .map(|p| key_rect.contains(p))
                .unwrap_or(false);

            let fill = if is_hovered && pointer_pressed {
                egui::Color32::from_rgb(80, 80, 160)
            } else {
                egui::Color32::from_rgb(40, 40, 40)
            };

            painter.rect_filled(key_rect, 2.0, fill);
            painter.rect_stroke(key_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK), egui::StrokeKind::Inside);

            // Black keys take priority over white keys for press detection
            if is_hovered && pointer_pressed {
                pressed_note = Some(midi_note);
            }
        }
    }

    // Generate note events based on pointer state
    let id = ui.id().with("keyboard_state");
    let prev_note: Option<u8> = ui.data(|d| d.get_temp(id)).unwrap_or(None);

    if pointer_pressed && rect.contains(pointer_pos.unwrap_or_default()) {
        if pressed_note != prev_note {
            // Release previous note if any
            if let Some(prev) = prev_note {
                events.push(NoteEvent::Off { note: prev });
            }
            // Press new note if any
            if let Some(note) = pressed_note {
                events.push(NoteEvent::On {
                    note,
                    velocity: 100,
                });
            }
        }
        ui.data_mut(|d| d.insert_temp(id, pressed_note));
    } else if pointer_released || (!pointer_pressed && prev_note.is_some()) {
        if let Some(prev) = prev_note {
            events.push(NoteEvent::Off { note: prev });
        }
        ui.data_mut(|d| d.insert_temp::<Option<u8>>(id, None));
    }

    events
}
