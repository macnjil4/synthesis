use std::collections::HashSet;

use eframe::egui;

use crate::midi::NoteEvent;
use crate::synth_ui::widgets::keyboard;

use super::synth_panel;

pub fn draw(ui: &mut egui::Ui, pressed_keys: &HashSet<u8>) -> Vec<NoteEvent> {
    let mut events = Vec::new();
    synth_panel(ui, "KEYBOARD", |ui| {
        events = keyboard::draw(ui, pressed_keys);
    });
    events
}
