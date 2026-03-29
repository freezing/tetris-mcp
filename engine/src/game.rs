use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::Board;
use crate::history::{Action, GameHistory, GameMetadata, MoveEffect, MoveRecord};
use crate::piece::{Direction, Piece, PieceType, RotateDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub current_piece: Piece,
    pub next_piece: PieceType,
    pub score: u64,
    pub lines_cleared: u32,
    pub level: u32,
    pub pieces_placed: u32,
    pub game_over: bool,
    pub game_over_reason: Option<GameOverReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOverReason {
    BlockOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveResult {
    Moved,
    Locked { lines_cleared: u32 },
    GameOver,
    Invalid,
}

pub struct Game {
    game_id: Uuid,
    seed: u64,
    width: usize,
    height: usize,
    state: GameState,
    rng: StdRng,
    bag: Vec<PieceType>,
    initial_state: GameState,
    move_log: Vec<MoveRecord>,
}

impl Game {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut bag = Vec::new();

        let first = Self::next_from_bag(&mut bag, &mut rng);
        let second = Self::next_from_bag(&mut bag, &mut rng);

        let state = GameState {
            board: Board::new(width, height),
            current_piece: Piece::new(first, width),
            next_piece: second,
            score: 0,
            lines_cleared: 0,
            level: 1,
            pieces_placed: 0,
            game_over: false,
            game_over_reason: None,
        };

        Self {
            game_id: Uuid::new_v4(),
            seed,
            width,
            height,
            initial_state: state.clone(),
            state,
            rng,
            bag,
            move_log: Vec::new(),
        }
    }

    pub fn game_id(&self) -> Uuid {
        self.game_id
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn move_piece(&mut self, direction: Direction) -> MoveResult {
        if self.state.game_over {
            return MoveResult::GameOver;
        }

        let candidate = self.state.current_piece.moved(direction);

        if self.state.board.is_valid_position(&candidate) {
            self.state.current_piece = candidate;
            self.log(Action::Move(direction), MoveEffect::PieceMoved { piece: candidate });
            MoveResult::Moved
        } else if direction == Direction::Down {
            let (result, effect) = self.lock_and_spawn();
            self.log(Action::Move(direction), effect);
            result
        } else {
            MoveResult::Invalid
        }
    }

    pub fn rotate(&mut self, dir: RotateDirection) -> MoveResult {
        if self.state.game_over {
            return MoveResult::GameOver;
        }

        let rotated = self.state.current_piece.rotated(dir);
        let kicks = self.state.current_piece.wall_kicks(dir);

        for &(dx, dy) in kicks {
            let candidate = Piece {
                x: rotated.x + dx,
                y: rotated.y + dy,
                ..rotated
            };
            if self.state.board.is_valid_position(&candidate) {
                self.state.current_piece = candidate;
                self.log(Action::Rotate(dir), MoveEffect::PieceMoved { piece: candidate });
                return MoveResult::Moved;
            }
        }

        MoveResult::Invalid
    }

    pub fn hard_drop(&mut self) -> MoveResult {
        if self.state.game_over {
            return MoveResult::GameOver;
        }

        let mut drop_distance: u64 = 0;
        loop {
            let candidate = self.state.current_piece.moved(Direction::Down);
            if self.state.board.is_valid_position(&candidate) {
                self.state.current_piece = candidate;
                drop_distance += 1;
            } else {
                break;
            }
        }

        self.state.score += drop_distance * 2;
        let (result, effect) = self.lock_and_spawn();
        self.log(Action::HardDrop, effect);
        result
    }

    pub fn history(&self) -> GameHistory {
        GameHistory {
            metadata: GameMetadata {
                game_id: self.game_id,
                seed: self.seed,
                board_width: self.width,
                board_height: self.height,
                final_score: self.state.score,
                lines_cleared: self.state.lines_cleared,
                pieces_placed: self.state.pieces_placed,
                total_moves: self.move_log.len() as u32,
                game_over: self.state.game_over,
            },
            initial_state: self.initial_state.clone(),
            moves: self.move_log.clone(),
        }
    }

    /// Attach reasoning to the most recent move record (typically a hard drop).
    pub fn annotate_last_move(&mut self, reasoning: String) {
        if let Some(last) = self.move_log.last_mut() {
            last.reasoning = Some(reasoning);
        }
    }

    fn log(&mut self, action: Action, effect: MoveEffect) {
        self.move_log.push(MoveRecord {
            move_index: self.move_log.len() as u32,
            action,
            effect,
            reasoning: None,
        });
    }

    fn lock_and_spawn(&mut self) -> (MoveResult, MoveEffect) {
        let locked_cells: Vec<(i32, i32)> = self.state.current_piece.cells().iter().copied().collect();
        let locked_type = self.state.current_piece.piece_type;

        self.state.board.lock_piece(&self.state.current_piece);
        self.state.pieces_placed += 1;

        let lines = self.state.board.clear_lines();
        self.state.lines_cleared += lines;
        self.state.score += Self::line_score(lines, self.state.level);
        self.state.level = self.state.lines_cleared / 10 + 1;

        let next_type = Self::next_from_bag(&mut self.bag, &mut self.rng);
        let new_piece = Piece::new(self.state.next_piece, self.width);
        self.state.next_piece = next_type;

        if !self.state.board.is_valid_position(&new_piece) {
            self.state.game_over = true;
            self.state.game_over_reason = Some(GameOverReason::BlockOut);
            let effect = MoveEffect::GameOver {
                locked_cells,
                locked_type,
                final_score: self.state.score,
            };
            return (MoveResult::GameOver, effect);
        }

        self.state.current_piece = new_piece;
        let effect = MoveEffect::PieceLocked {
            locked_cells,
            locked_type,
            lines_cleared: lines,
            score: self.state.score,
            new_piece,
            next_piece: next_type,
        };
        (MoveResult::Locked { lines_cleared: lines }, effect)
    }

    fn line_score(lines: u32, level: u32) -> u64 {
        let base = match lines {
            0 => 0,
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 800,
        };
        base * level as u64
    }

    fn next_from_bag(bag: &mut Vec<PieceType>, rng: &mut StdRng) -> PieceType {
        if bag.is_empty() {
            let mut new_bag = PieceType::ALL.to_vec();
            new_bag.shuffle(rng);
            *bag = new_bag;
        }
        bag.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_is_not_over() {
        let game = Game::new(10, 20, 42);
        assert!(!game.state().game_over);
        assert_eq!(game.state().score, 0);
        assert_eq!(game.state().lines_cleared, 0);
    }

    #[test]
    fn move_left_right() {
        let mut game = Game::new(10, 20, 42);
        let start_x = game.state().current_piece.x;

        assert_eq!(game.move_piece(Direction::Left), MoveResult::Moved);
        assert_eq!(game.state().current_piece.x, start_x - 1);

        assert_eq!(game.move_piece(Direction::Right), MoveResult::Moved);
        assert_eq!(game.state().current_piece.x, start_x);
    }

    #[test]
    fn move_down() {
        let mut game = Game::new(10, 20, 42);
        let start_y = game.state().current_piece.y;

        assert_eq!(game.move_piece(Direction::Down), MoveResult::Moved);
        assert_eq!(game.state().current_piece.y, start_y + 1);
    }

    #[test]
    fn hard_drop_locks_piece() {
        let mut game = Game::new(10, 20, 42);
        let result = game.hard_drop();
        match result {
            MoveResult::Locked { .. } => {}
            MoveResult::GameOver => {}
            other => panic!("Expected Locked or GameOver, got {other:?}"),
        }

        if !game.state().game_over {
            assert_eq!(game.state().pieces_placed, 1);
        }
    }

    #[test]
    fn rotation_works() {
        let mut game = Game::new(10, 20, 42);
        let original_rotation = game.state().current_piece.rotation;

        if game.rotate(RotateDirection::Cw) == MoveResult::Moved {
            assert_ne!(game.state().current_piece.rotation, original_rotation);
        }
    }

    #[test]
    fn wall_prevents_movement() {
        let mut game = Game::new(10, 20, 42);
        for _ in 0..20 {
            game.move_piece(Direction::Left);
        }
        let result = game.move_piece(Direction::Left);
        assert_eq!(result, MoveResult::Invalid);
    }

    #[test]
    fn scoring_for_lines() {
        assert_eq!(Game::line_score(0, 1), 0);
        assert_eq!(Game::line_score(1, 1), 100);
        assert_eq!(Game::line_score(2, 1), 300);
        assert_eq!(Game::line_score(3, 1), 500);
        assert_eq!(Game::line_score(4, 1), 800);
        assert_eq!(Game::line_score(1, 2), 200);
    }

    #[test]
    fn seven_bag_produces_all_pieces() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut bag = Vec::new();

        let mut seen = std::collections::HashSet::new();
        for _ in 0..7 {
            seen.insert(Game::next_from_bag(&mut bag, &mut rng));
        }
        assert_eq!(seen.len(), 7, "7-bag should produce all 7 piece types");
    }

    #[test]
    fn deterministic_with_same_seed() {
        let mut game1 = Game::new(10, 20, 123);
        let mut game2 = Game::new(10, 20, 123);

        for _ in 0..10 {
            let r1 = game1.hard_drop();
            let r2 = game2.hard_drop();
            assert_eq!(r1, r2);
        }

        assert_eq!(game1.state().score, game2.state().score);
    }

    #[test]
    fn invalid_moves_not_recorded() {
        let mut game = Game::new(10, 20, 42);
        for _ in 0..20 {
            game.move_piece(Direction::Left);
        }
        let before = game.history().moves.len();
        game.move_piece(Direction::Left); // invalid
        assert_eq!(game.history().moves.len(), before);
    }
}
