use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::GameState;
use crate::piece::{Direction, RotateDirection};

/// A single action taken during a game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
    Rotate(RotateDirection),
    HardDrop,
}

/// A record of one move in a game: the action taken and the resulting state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    pub move_index: u32,
    pub action: Action,
    pub state_after: GameState,
}

/// Summary metadata for a completed game.
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
