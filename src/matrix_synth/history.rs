use super::state::{COLS, ROWS};

type Grid = [[bool; COLS]; ROWS];

const MAX_HISTORY: usize = 20;

pub struct History {
    undo_stack: Vec<Grid>,
    redo_stack: Vec<Grid>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::with_capacity(MAX_HISTORY),
            redo_stack: Vec::with_capacity(MAX_HISTORY),
        }
    }

    /// Save state BEFORE modification
    pub fn push(&mut self, grid_before: Grid) {
        if self.undo_stack.len() >= MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(grid_before);
        self.redo_stack.clear();
    }

    /// Undo: restore previous state
    pub fn undo(&mut self, current_grid: &Grid) -> Option<Grid> {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(*current_grid);
            Some(previous)
        } else {
            None
        }
    }

    /// Redo: restore next state
    pub fn redo(&mut self) -> Option<Grid> {
        self.redo_stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_grid() -> Grid {
        [[false; COLS]; ROWS]
    }

    fn grid_with_cell(row: usize, col: usize) -> Grid {
        let mut g = empty_grid();
        g[row][col] = true;
        g
    }

    #[test]
    fn new_history_has_no_undo_redo() {
        let mut h = History::new();
        let grid = empty_grid();
        assert!(h.undo(&grid).is_none());
        assert!(h.redo().is_none());
    }

    #[test]
    fn push_and_undo() {
        let mut h = History::new();
        let before = empty_grid();
        let after = grid_with_cell(0, 0);
        h.push(before);
        let restored = h.undo(&after);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap(), before);
    }

    #[test]
    fn undo_then_redo() {
        let mut h = History::new();
        let before = empty_grid();
        let after = grid_with_cell(0, 0);
        h.push(before);
        let _ = h.undo(&after);
        let redone = h.redo();
        assert!(redone.is_some());
        assert_eq!(redone.unwrap(), after);
    }

    #[test]
    fn push_clears_redo() {
        let mut h = History::new();
        let g1 = empty_grid();
        let g2 = grid_with_cell(0, 0);
        let g3 = grid_with_cell(1, 1);
        h.push(g1);
        let _ = h.undo(&g2);
        // After undo, redo stack has g2. Push new state should clear redo.
        h.push(g3);
        assert!(h.redo().is_none());
    }

    #[test]
    fn max_history_limit() {
        let mut h = History::new();
        for i in 0..25 {
            let mut g = empty_grid();
            g[0][0] = i % 2 == 0;
            h.push(g);
        }
        // Should have at most MAX_HISTORY entries
        let mut count = 0;
        let grid = empty_grid();
        while h.undo(&grid).is_some() {
            count += 1;
        }
        assert!(count <= MAX_HISTORY);
    }
}
