use eframe::egui;
use eframe::egui::Key;

use super::history::History;
use super::state::*;

pub fn handle(ui: &mut egui::Ui, state: &mut TenoriState, history: &mut History) {
    let modifiers = ui.input(|i| i.modifiers);

    // ── Transport ──
    if ui.input(|i| i.key_pressed(Key::Space)) {
        state.toggle_play();
    }

    if !modifiers.ctrl
        && !modifiers.mac_cmd
        && ui.input(|i| i.key_pressed(Key::C))
    {
        let before = *state.active_grid();
        state.clear_grid();
        history.push(before);
    }

    // ── Mode toggle ──
    if !modifiers.ctrl
        && !modifiers.mac_cmd
        && ui.input(|i| i.key_pressed(Key::M))
    {
        state.mode = match state.mode {
            ChannelMode::Lead => ChannelMode::Drummer,
            ChannelMode::Drummer => ChannelMode::Bass,
            ChannelMode::Bass => ChannelMode::Lead,
        };
    }

    // ── BPM ──
    if modifiers.ctrl || modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
            state.bpm = (state.bpm + 5.0).min(240.0);
        }
        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
            state.bpm = (state.bpm - 5.0).max(40.0);
        }
    }

    // ── Draw modes ──
    if !modifiers.ctrl && !modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::D)) {
            state.draw_mode = DrawMode::Draw;
        }
        if ui.input(|i| i.key_pressed(Key::E)) {
            state.draw_mode = DrawMode::Erase;
        }
        if ui.input(|i| i.key_pressed(Key::T)) {
            state.draw_mode = DrawMode::Toggle;
        }
    }

    // ── Waveform ──
    if modifiers.ctrl || modifiers.mac_cmd {
        if ui.input(|i| i.key_pressed(Key::Num1)) {
            state.osc_waveform = Waveform::Sine;
        }
        if ui.input(|i| i.key_pressed(Key::Num2)) {
            state.osc_waveform = Waveform::Saw;
        }
        if ui.input(|i| i.key_pressed(Key::Num3)) {
            state.osc_waveform = Waveform::Square;
        }
        if ui.input(|i| i.key_pressed(Key::Num4)) {
            state.osc_waveform = Waveform::Tri;
        }
    }

    // ── Filter / LFO cycle ──
    if ui.input(|i| i.key_pressed(Key::Tab)) {
        if modifiers.shift {
            state.lfo_dest = state.lfo_dest.next();
        } else {
            state.filter_type = state.filter_type.next();
        }
    }

    // ── Undo / Redo ──
    if (modifiers.ctrl || modifiers.mac_cmd) && ui.input(|i| i.key_pressed(Key::Z)) {
        if modifiers.shift {
            if let Some(grid) = history.redo() {
                *state.active_grid_mut() = grid;
            }
        } else if let Some(grid) = history.undo(state.active_grid()) {
            *state.active_grid_mut() = grid;
        }
    }

    if (modifiers.ctrl || modifiers.mac_cmd)
        && ui.input(|i| i.key_pressed(Key::Y))
        && let Some(grid) = history.redo()
    {
        *state.active_grid_mut() = grid;
    }
}
