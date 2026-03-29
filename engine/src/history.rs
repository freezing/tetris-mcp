use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::GameState;
use crate::piece::{Direction, Piece, PieceType, RotateDirection};

/// A single action taken during a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
    Rotate(RotateDirection),
    HardDrop,
}

/// The effect of an action — compact delta instead of full board state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveEffect {
    /// Piece moved or rotated without locking.
    PieceMoved { piece: Piece },
    /// Piece locked into the board, lines optionally cleared, new piece spawned.
    PieceLocked {
        locked_cells: Vec<(i32, i32)>,
        locked_type: PieceType,
        lines_cleared: u32,
        score: u64,
        new_piece: Piece,
        next_piece: PieceType,
    },
    /// Game ended.
    GameOver {
        locked_cells: Vec<(i32, i32)>,
        locked_type: PieceType,
        final_score: u64,
    },
}

/// A record of one move: the action and its compact effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    pub move_index: u32,
    pub action: Action,
    pub effect: MoveEffect,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

/// Summary metadata for a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: Uuid,
    pub seed: u64,
    pub board_width: usize,
    pub board_height: usize,
    pub final_score: u64,
    pub lines_cleared: u32,
    pub pieces_placed: u32,
    pub total_moves: u32,
    pub game_over: bool,
}

/// Full recorded history of a game.
/// The viewer reconstructs any frame by starting from `initial_state`
/// and replaying `MoveEffect` deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistory {
    pub metadata: GameMetadata,
    pub initial_state: GameState,
    pub moves: Vec<MoveRecord>,
}

impl GameHistory {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("GameHistory serialization should not fail")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn game_records_history() {
        let mut game = Game::new(10, 20, 42);
        game.move_piece(Direction::Left);
        game.move_piece(Direction::Down);
        game.hard_drop();

        let history = game.history();
        assert_eq!(history.moves.len(), 3);
        assert_eq!(history.moves[0].move_index, 0);
        assert_eq!(history.moves[2].move_index, 2);
        assert!(matches!(history.moves[0].action, Action::Move(Direction::Left)));
        assert!(matches!(history.moves[2].action, Action::HardDrop));
    }

    #[test]
    fn move_records_are_compact() {
        let mut game = Game::new(10, 20, 42);
        game.move_piece(Direction::Left);

        let history = game.history();
        match &history.moves[0].effect {
            MoveEffect::PieceMoved { piece } => {
                // Should just store the piece position, not the whole board
                assert!(piece.x >= 0);
            }
            other => panic!("Expected PieceMoved, got {other:?}"),
        }
    }

    #[test]
    fn hard_drop_records_lock_effect() {
        let mut game = Game::new(10, 20, 42);
        game.hard_drop();

        let history = game.history();
        let last = history.moves.last().unwrap();
        match &last.effect {
            MoveEffect::PieceLocked { locked_cells, lines_cleared, .. } => {
                assert_eq!(locked_cells.len(), 4);
                assert!(*lines_cleared <= 4);
            }
            MoveEffect::GameOver { .. } => {} // also valid
            other => panic!("Expected PieceLocked or GameOver, got {other:?}"),
        }
    }

    #[test]
    fn history_roundtrips_through_json() {
        let mut game = Game::new(10, 20, 42);
        game.move_piece(Direction::Down);
        game.hard_drop();

        let history = game.history();
        let json = history.to_json();
        let restored = GameHistory::from_json(&json).unwrap();

        assert_eq!(restored.metadata.seed, 42);
        assert_eq!(restored.moves.len(), 2);
        assert_eq!(restored.metadata.game_id, history.metadata.game_id);
    }

    #[test]
    fn history_json_is_compact() {
        let mut game = Game::new(10, 20, 42);
        // Play a few moves
        for _ in 0..5 {
            game.move_piece(Direction::Down);
        }
        game.hard_drop();

        let history = game.history();
        let json = history.to_json();

        // A move-only record should be small — no full board dump
        // With 6 moves and an initial state, JSON should be reasonable
        // Initial state includes full board grid (~4KB for 10x20).
        // Move deltas should be small. 6 moves should add < 2KB.
        assert!(json.len() < 8000, "JSON too large: {} bytes", json.len());
    }

    #[test]
    fn history_has_correct_metadata() {
        let mut game = Game::new(10, 20, 99);
        game.hard_drop();
        game.hard_drop();

        let history = game.history();
        assert_eq!(history.metadata.seed, 99);
        assert_eq!(history.metadata.board_width, 10);
        assert_eq!(history.metadata.board_height, 20);
        assert_eq!(history.metadata.total_moves, 2);
    }
}
