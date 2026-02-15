use eframe::egui;

use crate::midi::NoteEvent;
use crate::synth_ui::widgets::pads;

use super::synth_panel;

pub fn draw(ui: &mut egui::Ui) -> Vec<NoteEvent> {
    let mut events = Vec::new();
    synth_panel(ui, "PADS", |ui| {
        events = pads::draw(ui);
    });
    events
}
