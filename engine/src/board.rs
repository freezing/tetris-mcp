use serde::{Deserialize, Serialize};

use crate::grid::Grid;
use crate::piece::Piece;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub grid: Grid,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    pub fn is_valid_position(&self, piece: &Piece) -> bool {
        piece
            .cells()
            .iter()
            .all(|&(x, y)| self.grid.in_bounds(x, y) && self.grid.is_empty(x, y))
    }

    pub fn lock_piece(&mut self, piece: &Piece) {
        for (x, y) in piece.cells() {
            if self.grid.in_bounds(x, y) {
                self.grid.set(x as usize, y as usize, Some(piece.piece_type));
            }
        }
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0;
        let mut write_row = self.grid.height() - 1;

        for read_row in (0..self.grid.height()).rev() {
            if self.grid.row_full(read_row) {
                cleared += 1;
            } else {
                if write_row != read_row {
                    self.grid.copy_row(read_row, write_row);
                }
                write_row = write_row.wrapping_sub(1);
            }
        }

        for row in 0..cleared as usize {
            self.grid.clear_row(row);
        }

        cleared
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::PieceType;

    const W: usize = 10;
    const H: usize = 20;

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(W, H);
        for y in 0..H {
            for x in 0..W {
                assert!(board.grid.get(x, y).is_none());
            }
        }
    }

    #[test]
    fn piece_at_spawn_is_valid() {
        let board = Board::new(W, H);
        let piece = Piece::new(PieceType::T, W);
        assert!(board.is_valid_position(&piece));
    }

    #[test]
    fn piece_out_of_bounds_is_invalid() {
        let board = Board::new(W, H);
        let mut piece = Piece::new(PieceType::I, W);
        piece.x = -5;
        assert!(!board.is_valid_position(&piece));
    }

    #[test]
    fn piece_collides_with_locked() {
        let mut board = Board::new(W, H);
        board.grid.set(3, 1, Some(PieceType::O));
        let piece = Piece::new(PieceType::T, W);
        assert!(!board.is_valid_position(&piece));
    }

    #[test]
    fn lock_piece_fills_cells() {
        let mut board = Board::new(W, H);
        let mut piece = Piece::new(PieceType::O, W);
        piece.y = 18;
        board.lock_piece(&piece);

        assert_eq!(board.grid.get(3, 18), Some(PieceType::O));
        assert_eq!(board.grid.get(4, 18), Some(PieceType::O));
        assert_eq!(board.grid.get(3, 19), Some(PieceType::O));
        assert_eq!(board.grid.get(4, 19), Some(PieceType::O));
    }

    #[test]
    fn clear_single_line() {
        let mut board = Board::new(W, H);
        for x in 0..W {
            board.grid.set(x, H - 1, Some(PieceType::I));
        }
        assert_eq!(board.clear_lines(), 1);
        for x in 0..W {
            assert!(board.grid.get(x, H - 1).is_none());
        }
    }

    #[test]
    fn clear_multiple_lines() {
        let mut board = Board::new(W, H);
        for y in (H - 4)..H {
            for x in 0..W {
                board.grid.set(x, y, Some(PieceType::I));
            }
        }
        assert_eq!(board.clear_lines(), 4);
    }

    #[test]
    fn clear_preserves_partial_rows() {
        let mut board = Board::new(W, H);
        for x in 0..W {
            board.grid.set(x, H - 1, Some(PieceType::I));
        }
        board.grid.set(0, H - 2, Some(PieceType::T));

        assert_eq!(board.clear_lines(), 1);
        assert_eq!(board.grid.get(0, H - 1), Some(PieceType::T));
        assert!(board.grid.get(1, H - 1).is_none());
    }
}
