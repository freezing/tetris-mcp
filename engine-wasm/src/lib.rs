use wasm_bindgen::prelude::*;
use engine::{Game, GameHistory};

/// WASM wrapper around the Tetris engine for the replay viewer.
/// Provides two modes:
/// 1. Replay: load a GameHistory JSON and step through it
/// 2. Live: create a new game and play interactively
#[wasm_bindgen]
pub struct TetrisReplay {
    history: GameHistory,
    game: Game,
    current_move: usize,
}

#[wasm_bindgen]
impl TetrisReplay {
    /// Load a game history from JSON and prepare for replay.
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<TetrisReplay, JsError> {
        let history = GameHistory::from_json(json).map_err(|e| JsError::new(&e.to_string()))?;
        let game = Game::new(
            history.metadata.board_width,
            history.metadata.board_height,
            history.metadata.seed,
        );
        Ok(TetrisReplay {
            history,
            game,
            current_move: 0,
        })
    }

    /// Total number of moves in the replay.
    pub fn total_moves(&self) -> u32 {
        self.history.moves.len() as u32
    }

    /// Current move index (0-based).
    pub fn current_move(&self) -> u32 {
        self.current_move as u32
    }

    /// Step forward one move. Returns false if already at the end.
    pub fn step_forward(&mut self) -> bool {
        if self.current_move >= self.history.moves.len() {
            return false;
        }

        let record = &self.history.moves[self.current_move];
        match &record.action {
            engine::Action::Move(dir) => { self.game.move_piece(*dir); }
            engine::Action::Rotate(dir) => { self.game.rotate(*dir); }
            engine::Action::HardDrop => { self.game.hard_drop(); }
        }

        self.current_move += 1;
        true
    }

    /// Reset to the beginning of the replay.
    pub fn reset(&mut self) {
        self.game = Game::new(
            self.history.metadata.board_width,
            self.history.metadata.board_height,
            self.history.metadata.seed,
        );
        self.current_move = 0;
    }

    /// Jump to a specific move index by replaying from the start.
    pub fn seek(&mut self, target_move: u32) {
        self.reset();
        let target = (target_move as usize).min(self.history.moves.len());
        for _ in 0..target {
            self.step_forward();
        }
    }

    /// Get the current game state as JSON.
    pub fn state_json(&self) -> String {
        serde_json::to_string(self.game.state()).unwrap_or_default()
    }

    /// Get the current board as a flat array of cell values.
    /// Each cell is either null or a piece type string.
    /// Row-major order: index = y * width + x.
    pub fn board_cells(&self) -> Vec<JsValue> {
        let board = &self.game.state().board;
        let mut cells = Vec::with_capacity(board.width() * board.height());
        for y in 0..board.height() {
            for x in 0..board.width() {
                match board.grid.get(x, y) {
                    Some(piece) => cells.push(JsValue::from_str(&format!("{piece:?}"))),
                    None => cells.push(JsValue::NULL),
                }
            }
        }
        cells
    }

    /// Get the current piece info as JSON.
    pub fn current_piece_json(&self) -> String {
        serde_json::to_string(&self.game.state().current_piece).unwrap_or_default()
    }

    /// Get the current score.
    pub fn score(&self) -> u64 {
        self.game.state().score
    }

    /// Get lines cleared.
    pub fn lines_cleared(&self) -> u32 {
        self.game.state().lines_cleared
    }

    /// Is the game over?
    pub fn is_game_over(&self) -> bool {
        self.game.state().game_over
    }

    /// Get board width.
    pub fn board_width(&self) -> u32 {
        self.game.state().board.width() as u32
    }

    /// Get board height.
    pub fn board_height(&self) -> u32 {
        self.game.state().board.height() as u32
    }

    /// Get metadata as JSON.
    pub fn metadata_json(&self) -> String {
        serde_json::to_string(&self.history.metadata).unwrap_or_default()
    }

    /// Get the reasoning for the most recent hard drop at or before the current move.
    /// Returns empty string if no reasoning is available.
    pub fn current_reasoning(&self) -> String {
        for i in (0..self.current_move).rev() {
            let record = &self.history.moves[i];
            if matches!(record.action, engine::Action::HardDrop) {
                return record.reasoning.clone().unwrap_or_default();
            }
        }
        String::new()
    }

    /// Peek at the next action type without executing it.
    /// Returns "Move", "Rotate", "HardDrop", or "" if at end.
    pub fn next_action_type(&self) -> String {
        if self.current_move >= self.history.moves.len() {
            return String::new();
        }
        match &self.history.moves[self.current_move].action {
            engine::Action::Move(_) => "Move".into(),
            engine::Action::Rotate(_) => "Rotate".into(),
            engine::Action::HardDrop => "HardDrop".into(),
        }
    }
}

