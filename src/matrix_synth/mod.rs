mod density_bar;
mod grid;
mod header;
mod history;
pub mod panels;
mod shortcuts;
pub mod state;
mod theme;
mod transport;
pub mod widgets;

use eframe::egui;

pub use state::MatrixState;

use history::History;
use state::{ChannelMode, COLS};
use theme::Theme;

#[allow(dead_code)]
pub struct MatrixSynth {
    state: MatrixState,
    history: History,
    prev_play_col: i32,
}

impl MatrixSynth {
    pub fn new() -> Self {
        Self {
            state: MatrixState::default(),
            history: History::new(),
            prev_play_col: -1,
        }
    }

    /// Main display method — call from the egui loop.
    pub fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Background
        let screen = ui.max_rect();
        ui.painter().rect_filled(screen, 0.0, Theme::BG);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), 300.0)),
            0.0,
            Theme::BG_GRADIENT_TOP,
        );

        // Update playhead
        self.update_playhead(ctx);

        // Keyboard shortcuts
        shortcuts::handle(ui, &mut self.state, &mut self.history);

        // Layout
        ui.vertical(|ui| {
            // Header
            header::draw(ui, &self.state);
            ui.add_space(14.0);

            // Body: sidebar left + main area right
            ui.horizontal_top(|ui| {
                // ── Left sidebar ──
                ui.allocate_ui_with_layout(
                    egui::vec2(Theme::SIDEBAR_WIDTH, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_width(Theme::SIDEBAR_WIDTH);
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                match self.state.mode {
                                    ChannelMode::Lead => {
                                        panels::oscillator::draw(ui, &mut self.state);
                                        panels::envelope::draw(ui, &mut self.state);
                                        panels::filter::draw(ui, &mut self.state);
                                        panels::lfo::draw(ui, &mut self.state);
                                        panels::effects::draw(ui, &mut self.state);
                                        panels::scale::draw(ui, &mut self.state);
                                    }
                                    ChannelMode::Drummer => {
                                        panels::drum_kit::draw(ui, &mut self.state);
                                        panels::effects::draw(ui, &mut self.state);
                                    }
                                    ChannelMode::Bass => {
                                        panels::bass_preset::draw(ui, &mut self.state);
                                        panels::effects::draw(ui, &mut self.state);
                                        panels::scale::draw(ui, &mut self.state);
                                    }
                                }
                            });
                    },
                );

                ui.add_space(12.0);

                // ── Main area ──
                ui.vertical(|ui| {
                    transport::draw(ui, &mut self.state, &mut self.history);
                    ui.add_space(8.0);
                    grid::draw(ui, &mut self.state, &mut self.history);
                    ui.add_space(8.0);
                    density_bar::draw(ui, &self.state);
                });
            });

            ui.add_space(12.0);

            // Footer
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(
                        "CLICK CELLS TO ACTIVATE \u{00b7} SPACE PLAY/PAUSE \u{00b7} C CLEAR \u{00b7} M MODE",
                    )
                    .size(9.0)
                    .color(Theme::TEXT_DIM),
                );
            });
        });

        // Continuous repaint while playing
        if self.state.is_playing {
            ctx.request_repaint();
        }
    }

    fn update_playhead(&mut self, ctx: &egui::Context) {
        if !self.state.is_playing {
            return;
        }

        let dt = ctx.input(|i| i.stable_dt) as f64;
        self.state.elapsed_secs += dt;

        // Step duration = 1/16 note
        let step_secs = 60.0 / self.state.bpm as f64 / 4.0;

        while self.state.elapsed_secs >= step_secs {
            self.state.elapsed_secs -= step_secs;
            self.state.play_col = (self.state.play_col + 1) % COLS as i32;
        }
    }

    // ── Public API ──

    #[allow(dead_code)]
    pub fn state(&self) -> &MatrixState {
        &self.state
    }

    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut MatrixState {
        &mut self.state
    }

    #[allow(dead_code)]
    pub fn active_note_names(&self) -> Vec<&'static str> {
        self.state.active_note_names()
    }

    #[allow(dead_code)]
    pub fn active_rows(&self) -> Vec<usize> {
        self.state.active_rows()
    }

    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.state.is_playing
    }

    #[allow(dead_code)]
    pub fn current_step(&self) -> i32 {
        self.state.play_col
    }

    /// Check if the playhead moved to a new column since last call.
    /// Returns Some(new_col) if it changed, None otherwise.
    #[allow(dead_code)]
    pub fn step_changed(&mut self) -> Option<i32> {
        let current = self.state.play_col;
        if current != self.prev_play_col {
            self.prev_play_col = current;
            Some(current)
        } else {
            None
        }
    }
}
