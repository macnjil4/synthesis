use std::sync::mpsc;

/// MIDI note events sent from the MIDI thread to the GUI thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteEvent {
    On { note: u8, velocity: u8 },
    Off { note: u8 },
}

impl NoteEvent {
    /// Parse a raw MIDI message into a NoteEvent, if applicable.
    pub fn from_midi(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }
        let status = data[0] & 0xF0;
        let note = data[1];
        let velocity = data[2];

        match status {
            0x90 if velocity > 0 => Some(NoteEvent::On { note, velocity }),
            0x90 => Some(NoteEvent::Off { note }), // velocity 0 = note off
            0x80 => Some(NoteEvent::Off { note }),
            _ => None,
        }
    }
}

/// Manages a MIDI input connection and forwards note events via a channel.
pub struct MidiHandler {
    _connection: Option<midir::MidiInputConnection<()>>,
    receiver: Option<mpsc::Receiver<NoteEvent>>,
    port_names: Vec<String>,
    selected_port: Option<usize>,
}

impl MidiHandler {
    pub fn new() -> Self {
        let port_names = Self::scan_ports();
        Self {
            _connection: None,
            receiver: None,
            port_names,
            selected_port: None,
        }
    }

    fn scan_ports() -> Vec<String> {
        let Ok(midi_in) = midir::MidiInput::new("synthesis-scan") else {
            return Vec::new();
        };
        midi_in
            .ports()
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect()
    }

    /// Re-scan available MIDI input ports.
    pub fn refresh_ports(&mut self) {
        self.port_names = Self::scan_ports();
    }

    pub fn port_names(&self) -> &[String] {
        &self.port_names
    }

    pub fn selected_port(&self) -> Option<usize> {
        self.selected_port
    }

    pub fn is_connected(&self) -> bool {
        self._connection.is_some()
    }

    /// Connect to a MIDI input port by index.
    /// Optionally pass an egui::Context to request repaint on MIDI events.
    pub fn connect(&mut self, port_idx: usize, ctx: Option<eframe::egui::Context>) {
        self.disconnect();

        let Ok(midi_in) = midir::MidiInput::new("synthesis") else {
            return;
        };

        let ports = midi_in.ports();
        let Some(port) = ports.get(port_idx) else {
            return;
        };

        let (tx, rx) = mpsc::channel();
        let connection = midi_in
            .connect(
                port,
                "synthesis-input",
                move |_timestamp, data, _| {
                    if let Some(event) = NoteEvent::from_midi(data) {
                        let _ = tx.send(event);
                        if let Some(ctx) = &ctx {
                            ctx.request_repaint();
                        }
                    }
                },
                (),
            )
            .ok();

        if connection.is_some() {
            self.selected_port = Some(port_idx);
        }
        self._connection = connection;
        self.receiver = Some(rx);
    }

    /// Disconnect from the current MIDI port.
    pub fn disconnect(&mut self) {
        self._connection = None;
        self.receiver = None;
        self.selected_port = None;
    }

    /// Try to receive the next pending NoteEvent (non-blocking).
    pub fn try_recv(&self) -> Option<NoteEvent> {
        self.receiver.as_ref()?.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_note_on() {
        let event = NoteEvent::from_midi(&[0x90, 60, 100]);
        assert_eq!(event, Some(NoteEvent::On { note: 60, velocity: 100 }));
    }

    #[test]
    fn parse_note_off() {
        let event = NoteEvent::from_midi(&[0x80, 60, 64]);
        assert_eq!(event, Some(NoteEvent::Off { note: 60 }));
    }

    #[test]
    fn parse_note_on_zero_velocity_is_off() {
        let event = NoteEvent::from_midi(&[0x90, 60, 0]);
        assert_eq!(event, Some(NoteEvent::Off { note: 60 }));
    }

    #[test]
    fn parse_note_on_channel_agnostic() {
        // Note on, channel 5
        let event = NoteEvent::from_midi(&[0x95, 72, 80]);
        assert_eq!(event, Some(NoteEvent::On { note: 72, velocity: 80 }));
    }

    #[test]
    fn parse_invalid_message_too_short() {
        assert_eq!(NoteEvent::from_midi(&[0x90, 60]), None);
        assert_eq!(NoteEvent::from_midi(&[0x90]), None);
        assert_eq!(NoteEvent::from_midi(&[]), None);
    }

    #[test]
    fn parse_non_note_message() {
        // Control change
        let event = NoteEvent::from_midi(&[0xB0, 1, 64]);
        assert_eq!(event, None);
    }

    #[test]
    fn midi_handler_starts_disconnected() {
        let handler = MidiHandler::new();
        assert!(!handler.is_connected());
        assert_eq!(handler.selected_port(), None);
        assert_eq!(handler.try_recv(), None);
    }
}
