use eframe::egui;
use eframe::egui::{Color32, Stroke};

use super::history::History;
use super::state::{ChannelMode, DrawMode, MatrixState, COLS, ROWS};
use super::theme::Theme;
use super::widgets;

#[allow(clippy::needless_range_loop)]
pub fn draw(ui: &mut egui::Ui, state: &mut MatrixState, history: &mut History) {
    let cell = Theme::CELL_SIZE;
    let gap = Theme::CELL_GAP;
    let step = cell + gap;
    let ctrl_width = Theme::ROW_CTRL_GAP + Theme::MUTE_BTN_SIZE + Theme::CTRL_GAP + Theme::VOL_BAR_WIDTH;
    let grid_width = Theme::NOTE_LABEL_WIDTH + COLS as f32 * step + ctrl_width;
    let grid_height = Theme::COL_HEADER_HEIGHT + ROWS as f32 * step;

    widgets::panel::synth_panel_no_title(ui, |ui| {
        let (total_rect, _) =
            ui.allocate_exact_size(egui::vec2(grid_width, grid_height), egui::Sense::hover());
        let painter = ui.painter_at(total_rect);
        let origin = total_rect.min;

        // ── Column numbers ──
        for col in 0..COLS {
            let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step + cell / 2.0;
            let y = origin.y + Theme::COL_HEADER_HEIGHT / 2.0;
            let is_active = state.is_playing && state.play_col == col as i32;
            painter.text(
                egui::pos2(x, y),
                egui::Align2::CENTER_CENTER,
                format!("{}", col + 1),
                egui::FontId::proportional(8.0),
                if is_active {
                    Theme::ACCENT_LIGHT
                } else {
                    Theme::TEXT_DIM
                },
            );
        }

        // ── Playhead column background ──
        if state.is_playing && state.play_col >= 0 {
            let col = state.play_col as usize;
            let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step - 1.0;
            let y = origin.y + Theme::COL_HEADER_HEIGHT;
            let h = ROWS as f32 * step;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell + 2.0, h)),
                4.0,
                Theme::PLAYHEAD_BG,
            );
        }

        // ── Row labels (notes or drum names) ──
        let labels = state.row_labels();
        for row in 0..ROWS {
            let label = labels[row];
            let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step + cell / 2.0;
            let x = origin.x + Theme::NOTE_LABEL_WIDTH - 4.0;
            let color = match state.mode {
                ChannelMode::Lead | ChannelMode::Bass => {
                    if label.contains('#') {
                        Theme::ACCENT
                    } else {
                        Theme::TEXT
                    }
                }
                ChannelMode::Drummer => Theme::TEXT,
            };
            painter.text(
                egui::pos2(x, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::proportional(9.0),
                color,
            );
        }

        // ── Cells: interaction pass ──
        let grid_before = *state.active_grid();
        let mut grid_changed = false;

        // Track hover state per cell
        let mut hovered = [[false; COLS]; ROWS];

        for row in 0..ROWS {
            for col in 0..COLS {
                let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step;
                let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;
                let cell_rect =
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));

                let cell_id = ui.id().with(("cell", row, col));
                let response =
                    ui.interact(cell_rect, cell_id, egui::Sense::click_and_drag());

                hovered[row][col] = response.hovered();

                let should_apply =
                    response.clicked() || (response.dragged() && response.hovered());

                if should_apply {
                    let draw_mode = state.draw_mode;
                    let grid = state.active_grid_mut();
                    match draw_mode {
                        DrawMode::Toggle => {
                            if response.clicked() {
                                grid[row][col] = !grid[row][col];
                                grid_changed = true;
                            }
                        }
                        DrawMode::Draw => {
                            if !grid[row][col] {
                                grid[row][col] = true;
                                grid_changed = true;
                            }
                        }
                        DrawMode::Erase => {
                            if grid[row][col] {
                                grid[row][col] = false;
                                grid_changed = true;
                            }
                        }
                    }
                }

                // Modifier clicks
                if response.clicked() {
                    let modifiers = ui.input(|i| i.modifiers);
                    if modifiers.shift {
                        state.toggle_row(row);
                        grid_changed = true;
                    } else if modifiers.ctrl || modifiers.mac_cmd {
                        state.toggle_col(col);
                        grid_changed = true;
                    }
                }
            }
        }

        // Pass 0: Ghost overlay (other modes' cells in transparency)
        let ghosts: Vec<(&[[bool; COLS]; ROWS], Color32)> = match state.mode {
            ChannelMode::Lead => vec![
                (&state.drum_grid, Theme::GHOST_DRUM),
                (&state.bass_grid, Theme::GHOST_BASS),
            ],
            ChannelMode::Drummer => vec![
                (&state.grid, Theme::GHOST_LEAD),
                (&state.bass_grid, Theme::GHOST_BASS),
            ],
            ChannelMode::Bass => vec![
                (&state.grid, Theme::GHOST_LEAD),
                (&state.drum_grid, Theme::GHOST_DRUM),
            ],
        };
        for (ghost_grid, ghost_color) in &ghosts {
            for row in 0..ROWS {
                for col in 0..COLS {
                    if ghost_grid[row][col] {
                        let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step;
                        let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;
                        let cell_rect =
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));
                        painter.rect_filled(cell_rect, Theme::CELL_ROUNDING, *ghost_color);
                    }
                }
            }
        }

        // Pass 1: Draw glows for ON and HIT cells (mode-aware colors)
        for row in 0..ROWS {
            for col in 0..COLS {
                let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step;
                let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;
                let cell_rect =
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));

                let is_on = state.active_grid()[row][col];
                let is_playhead = state.is_playing && state.play_col == col as i32;
                let is_hit = is_on && is_playhead;

                if is_hit {
                    let glow = match state.mode {
                        ChannelMode::Lead => Theme::LEAD_CELL_HIT_GLOW,
                        ChannelMode::Drummer => Theme::CELL_HIT_GLOW,
                        ChannelMode::Bass => Theme::BASS_CELL_HIT_GLOW,
                    };
                    painter.rect_filled(
                        cell_rect.expand(3.0),
                        Theme::CELL_ROUNDING + 3.0,
                        glow,
                    );
                } else if is_on {
                    let subtle_glow = match state.mode {
                        ChannelMode::Lead => Color32::from_rgba_premultiplied(234, 179, 8, 20),
                        ChannelMode::Drummer => Color32::from_rgba_premultiplied(124, 58, 237, 20),
                        ChannelMode::Bass => Color32::from_rgba_premultiplied(236, 72, 153, 20),
                    };
                    painter.rect_filled(
                        cell_rect.expand(1.5),
                        Theme::CELL_ROUNDING + 1.5,
                        subtle_glow,
                    );
                }
            }
        }

        // Pass 2: Draw all cells (mode-aware colors)
        for row in 0..ROWS {
            for col in 0..COLS {
                let x = origin.x + Theme::NOTE_LABEL_WIDTH + col as f32 * step;
                let y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;
                let cell_rect =
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));

                let is_hovered = hovered[row][col];
                let is_on = state.active_grid()[row][col];
                let is_playhead = state.is_playing && state.play_col == col as i32;
                let is_hit = is_on && is_playhead;

                let (bg, border_color) = match state.mode {
                    ChannelMode::Lead => {
                        if is_hit {
                            (Theme::LEAD_CELL_HIT, Theme::LEAD_ACCENT_LIGHT)
                        } else if is_on && is_hovered {
                            (Theme::LEAD_CELL_ON_HOVER, Theme::LEAD_ACCENT)
                        } else if is_on {
                            (Theme::LEAD_CELL_ON, Theme::LEAD_ACCENT)
                        } else if is_playhead {
                            (Theme::CELL_PLAYHEAD, Theme::CELL_PLAYHEAD_BORDER)
                        } else if is_hovered {
                            (Theme::CELL_OFF_HOVER, Theme::BORDER)
                        } else {
                            (Theme::CELL_OFF, Theme::BORDER)
                        }
                    }
                    ChannelMode::Drummer => {
                        if is_hit {
                            (Theme::CELL_HIT, Theme::ACCENT_LIGHT)
                        } else if is_on && is_hovered {
                            (Theme::CELL_ON_HOVER, Theme::ACCENT)
                        } else if is_on {
                            (Theme::CELL_ON, Theme::ACCENT)
                        } else if is_playhead {
                            (Theme::CELL_PLAYHEAD, Theme::CELL_PLAYHEAD_BORDER)
                        } else if is_hovered {
                            (Theme::CELL_OFF_HOVER, Theme::BORDER)
                        } else {
                            (Theme::CELL_OFF, Theme::BORDER)
                        }
                    }
                    ChannelMode::Bass => {
                        if is_hit {
                            (Theme::BASS_CELL_HIT, Theme::BASS_ACCENT_LIGHT)
                        } else if is_on && is_hovered {
                            (Theme::BASS_CELL_ON_HOVER, Theme::BASS_ACCENT)
                        } else if is_on {
                            (Theme::BASS_CELL_ON, Theme::BASS_ACCENT)
                        } else if is_playhead {
                            (Theme::CELL_PLAYHEAD, Theme::CELL_PLAYHEAD_BORDER)
                        } else if is_hovered {
                            (Theme::CELL_OFF_HOVER, Theme::BORDER)
                        } else {
                            (Theme::CELL_OFF, Theme::BORDER)
                        }
                    }
                };

                painter.rect_filled(cell_rect, Theme::CELL_ROUNDING, bg);
                painter.rect_stroke(
                    cell_rect,
                    Theme::CELL_ROUNDING,
                    Stroke::new(1.0, border_color),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // ── Row controls (mute button + volume bar) ──
        let cells_right = origin.x + Theme::NOTE_LABEL_WIDTH + COLS as f32 * step;
        let mute_x = cells_right + Theme::ROW_CTRL_GAP;

        for row in 0..ROWS {
            let row_y = origin.y + Theme::COL_HEADER_HEIGHT + row as f32 * step;

            // ── Mute button ──
            let mute_rect = egui::Rect::from_min_size(
                egui::pos2(mute_x, row_y + (cell - Theme::MUTE_BTN_SIZE) / 2.0),
                egui::vec2(Theme::MUTE_BTN_SIZE, Theme::MUTE_BTN_SIZE),
            );
            let mute_id = ui.id().with(("mute", row));
            let mute_resp = ui.interact(mute_rect, mute_id, egui::Sense::click());
            if mute_resp.clicked() {
                let mutes = state.active_row_mute_mut();
                mutes[row] = !mutes[row];
            }
            let is_muted = state.active_row_mute()[row];
            let mute_color = if is_muted { Theme::MUTE_ON } else { Theme::MUTE_OFF };
            painter.rect_filled(mute_rect, 3.0, mute_color);
            painter.text(
                mute_rect.center(),
                egui::Align2::CENTER_CENTER,
                "M",
                egui::FontId::proportional(10.0),
                if is_muted { Theme::TEXT_WHITE } else { Theme::TEXT_DIM },
            );

            // ── Volume bar ──
            let vol_x = mute_x + Theme::MUTE_BTN_SIZE + Theme::CTRL_GAP;
            let vol_rect = egui::Rect::from_min_size(
                egui::pos2(vol_x, row_y + (cell - Theme::VOL_BAR_HEIGHT) / 2.0),
                egui::vec2(Theme::VOL_BAR_WIDTH, Theme::VOL_BAR_HEIGHT),
            );
            let vol_id = ui.id().with(("vol", row));
            let vol_resp = ui.interact(vol_rect, vol_id, egui::Sense::click_and_drag());
            if (vol_resp.clicked() || vol_resp.dragged())
                && let Some(pos) = vol_resp.interact_pointer_pos()
            {
                let t = ((pos.x - vol_rect.left()) / vol_rect.width()).clamp(0.0, 1.0);
                state.active_row_volume_mut()[row] = t;
            }
            let vol = state.active_row_volume()[row];
            painter.rect_filled(vol_rect, 2.0, Theme::VOL_BAR_BG);
            let fill_rect = egui::Rect::from_min_size(
                vol_rect.min,
                egui::vec2(vol_rect.width() * vol, vol_rect.height()),
            );
            painter.rect_filled(fill_rect, 2.0, Theme::VOL_BAR_FILL);

            // ── Muted-row overlay (dim cell row) ──
            if is_muted {
                let overlay_rect = egui::Rect::from_min_size(
                    egui::pos2(origin.x + Theme::NOTE_LABEL_WIDTH, row_y),
                    egui::vec2(COLS as f32 * step, cell),
                );
                painter.rect_filled(overlay_rect, 0.0, Theme::MUTED_ROW_OVERLAY);
            }
        }

        // Save to history if grid changed
        if grid_changed {
            history.push(grid_before);
        }
    });
}
