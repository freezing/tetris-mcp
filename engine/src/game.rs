use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::board::Board;
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
    state: GameState,
    width: usize,
    rng: StdRng,
    bag: Vec<PieceType>,
}

impl Game {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut bag = Vec::new();

        let first = Self::next_from_bag(&mut bag, &mut rng);
        let second = Self::next_from_bag(&mut bag, &mut rng);

        Self {
            state: GameState {
                board: Board::new(width, height),
                current_piece: Piece::new(first, width),
                next_piece: second,
                score: 0,
                lines_cleared: 0,
                level: 1,
                pieces_placed: 0,
                game_over: false,
                game_over_reason: None,
            },
            width,
            rng,
            bag,
        }
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
            MoveResult::Moved
        } else if direction == Direction::Down {
            self.lock_and_spawn()
        } else {
            MoveResult::Invalid
        }
    }

    pub fn rotate(&mut self, dir: RotateDirection) -> MoveResult {
        self.try_rotate(dir)
    }

    pub fn hard_drop(&mut self) -> MoveResult {
        if self.state.game_over {
            return MoveResult::GameOver;
        }

        let mut drop_distance = 0;
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
        self.lock_and_spawn()
    }

    fn try_rotate(&mut self, dir: RotateDirection) -> MoveResult {
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
                return MoveResult::Moved;
            }
        }

        MoveResult::Invalid
    }

    fn lock_and_spawn(&mut self) -> MoveResult {
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
            return MoveResult::GameOver;
        }

        self.state.current_piece = new_piece;
        MoveResult::Locked { lines_cleared: lines }
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

        // Current piece should be different (next piece spawned)
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
        // Move all the way left
        for _ in 0..20 {
            game.move_piece(Direction::Left);
        }
        // One more should be invalid
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
}
