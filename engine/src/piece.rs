use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceType {
    pub const ALL: [PieceType; 7] = [
        PieceType::I,
        PieceType::O,
        PieceType::T,
        PieceType::S,
        PieceType::Z,
        PieceType::J,
        PieceType::L,
    ];

    /// Returns the cells occupied by this piece in the given rotation,
    /// relative to the piece's origin (0,0).
    fn cells(self, rotation: Rotation) -> PieceCells {
        let raw = match (self, rotation) {
            // I piece
            (PieceType::I, Rotation::North) => [(0, 1), (1, 1), (2, 1), (3, 1)],
            (PieceType::I, Rotation::East) => [(2, 0), (2, 1), (2, 2), (2, 3)],
            (PieceType::I, Rotation::South) => [(0, 2), (1, 2), (2, 2), (3, 2)],
            (PieceType::I, Rotation::West) => [(1, 0), (1, 1), (1, 2), (1, 3)],
            // O piece (same in all rotations)
            (PieceType::O, _) => [(0, 0), (1, 0), (0, 1), (1, 1)],
            // T piece
            (PieceType::T, Rotation::North) => [(1, 0), (0, 1), (1, 1), (2, 1)],
            (PieceType::T, Rotation::East) => [(1, 0), (1, 1), (2, 1), (1, 2)],
            (PieceType::T, Rotation::South) => [(0, 1), (1, 1), (2, 1), (1, 2)],
            (PieceType::T, Rotation::West) => [(1, 0), (0, 1), (1, 1), (1, 2)],
            // S piece
            (PieceType::S, Rotation::North) => [(1, 0), (2, 0), (0, 1), (1, 1)],
            (PieceType::S, Rotation::East) => [(1, 0), (1, 1), (2, 1), (2, 2)],
            (PieceType::S, Rotation::South) => [(1, 1), (2, 1), (0, 2), (1, 2)],
            (PieceType::S, Rotation::West) => [(0, 0), (0, 1), (1, 1), (1, 2)],
            // Z piece
            (PieceType::Z, Rotation::North) => [(0, 0), (1, 0), (1, 1), (2, 1)],
            (PieceType::Z, Rotation::East) => [(2, 0), (1, 1), (2, 1), (1, 2)],
            (PieceType::Z, Rotation::South) => [(0, 1), (1, 1), (1, 2), (2, 2)],
            (PieceType::Z, Rotation::West) => [(1, 0), (0, 1), (1, 1), (0, 2)],
            // J piece
            (PieceType::J, Rotation::North) => [(0, 0), (0, 1), (1, 1), (2, 1)],
            (PieceType::J, Rotation::East) => [(1, 0), (2, 0), (1, 1), (1, 2)],
            (PieceType::J, Rotation::South) => [(0, 1), (1, 1), (2, 1), (2, 2)],
            (PieceType::J, Rotation::West) => [(1, 0), (1, 1), (0, 2), (1, 2)],
            // L piece
            (PieceType::L, Rotation::North) => [(2, 0), (0, 1), (1, 1), (2, 1)],
            (PieceType::L, Rotation::East) => [(1, 0), (1, 1), (1, 2), (2, 2)],
            (PieceType::L, Rotation::South) => [(0, 1), (1, 1), (2, 1), (0, 2)],
            (PieceType::L, Rotation::West) => [(0, 0), (1, 0), (1, 1), (1, 2)],
        };
        PieceCells(raw)
    }
}

/// The 4 cells occupied by a tetromino, as (x, y) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceCells([(i32, i32); 4]);

impl PieceCells {
    pub fn iter(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.0.iter()
    }

    pub fn contains(&self, point: &(i32, i32)) -> bool {
        self.0.contains(point)
    }

    pub fn as_slice(&self) -> &[(i32, i32)] {
        &self.0
    }
}

impl IntoIterator for PieceCells {
    type Item = (i32, i32);
    type IntoIter = std::array::IntoIter<(i32, i32), 4>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}

impl Rotation {
    fn apply(self, dir: RotateDirection) -> Rotation {
        match (self, dir) {
            (Rotation::North, RotateDirection::Cw) => Rotation::East,
            (Rotation::East, RotateDirection::Cw) => Rotation::South,
            (Rotation::South, RotateDirection::Cw) => Rotation::West,
            (Rotation::West, RotateDirection::Cw) => Rotation::North,
            (Rotation::North, RotateDirection::Ccw) => Rotation::West,
            (Rotation::East, RotateDirection::Ccw) => Rotation::North,
            (Rotation::South, RotateDirection::Ccw) => Rotation::East,
            (Rotation::West, RotateDirection::Ccw) => Rotation::South,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotateDirection {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub rotation: Rotation,
    pub x: i32,
    pub y: i32,
}

impl Piece {
    pub fn new(piece_type: PieceType, board_width: usize) -> Self {
        Self {
            piece_type,
            rotation: Rotation::North,
            x: (board_width as i32 - 4) / 2,
            y: 0,
        }
    }

    pub fn cells(&self) -> PieceCells {
        let local = self.piece_type.cells(self.rotation);
        PieceCells(local.0.map(|(dx, dy)| (self.x + dx, self.y + dy)))
    }

    pub fn moved(&self, direction: Direction) -> Self {
        let (dx, dy) = match direction {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
        };
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..*self
        }
    }

    pub fn rotated(&self, dir: RotateDirection) -> Self {
        Self {
            rotation: self.rotation.apply(dir),
            ..*self
        }
    }

    /// SRS wall kick offsets for rotating in the given direction.
    pub fn wall_kicks(&self, dir: RotateDirection) -> &'static [(i32, i32)] {
        let from = self.rotation;
        let to = from.apply(dir);
        if self.piece_type == PieceType::I {
            srs_kicks_i(from, to)
        } else if self.piece_type == PieceType::O {
            &[(0, 0)]
        } else {
            srs_kicks_jlstz(from, to)
        }
    }
}

// SRS kick tables — only adjacent rotation transitions are possible,
// guaranteed by Rotation::apply only producing CW/CCW neighbors.

fn srs_kicks_jlstz(from: Rotation, to: Rotation) -> &'static [(i32, i32)] {
    match (from, to) {
        (Rotation::North, Rotation::East) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (Rotation::East, Rotation::North) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (Rotation::East, Rotation::South) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (Rotation::South, Rotation::East) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        (Rotation::South, Rotation::West) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (Rotation::West, Rotation::South) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (Rotation::West, Rotation::North) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (Rotation::North, Rotation::West) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        _ => unreachable!(),
    }
}

fn srs_kicks_i(from: Rotation, to: Rotation) -> &'static [(i32, i32)] {
    match (from, to) {
        (Rotation::North, Rotation::East) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (Rotation::East, Rotation::North) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (Rotation::East, Rotation::South) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        (Rotation::South, Rotation::East) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (Rotation::South, Rotation::West) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
        (Rotation::West, Rotation::South) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
        (Rotation::West, Rotation::North) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
        (Rotation::North, Rotation::West) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_spawns_centered() {
        let piece = Piece::new(PieceType::T, 10);
        assert_eq!(piece.x, 3);
        assert_eq!(piece.y, 0);
        assert_eq!(piece.rotation, Rotation::North);

        let piece = Piece::new(PieceType::T, 6);
        assert_eq!(piece.x, 1);
    }

    #[test]
    fn piece_moves_correctly() {
        let piece = Piece::new(PieceType::T, 10);
        assert_eq!(piece.moved(Direction::Left).x, 2);
        assert_eq!(piece.moved(Direction::Right).x, 4);
        assert_eq!(piece.moved(Direction::Down).y, 1);
    }

    #[test]
    fn rotation_cycles() {
        assert_eq!(Rotation::North.apply(RotateDirection::Cw), Rotation::East);
        assert_eq!(Rotation::East.apply(RotateDirection::Cw), Rotation::South);
        assert_eq!(Rotation::South.apply(RotateDirection::Cw), Rotation::West);
        assert_eq!(Rotation::West.apply(RotateDirection::Cw), Rotation::North);

        assert_eq!(Rotation::North.apply(RotateDirection::Ccw), Rotation::West);
        assert_eq!(Rotation::West.apply(RotateDirection::Ccw), Rotation::South);
    }

    #[test]
    fn all_pieces_have_4_cells() {
        for piece_type in PieceType::ALL {
            for rotation in [Rotation::North, Rotation::East, Rotation::South, Rotation::West] {
                assert_eq!(piece_type.cells(rotation).as_slice().len(), 4, "{piece_type:?} {rotation:?}");
            }
        }
    }

    #[test]
    fn o_piece_same_in_all_rotations() {
        let north = PieceType::O.cells(Rotation::North);
        let east = PieceType::O.cells(Rotation::East);
        let south = PieceType::O.cells(Rotation::South);
        let west = PieceType::O.cells(Rotation::West);
        assert_eq!(north, east);
        assert_eq!(east, south);
        assert_eq!(south, west);
    }

    #[test]
    fn piece_cells_are_absolute() {
        let piece = Piece::new(PieceType::I, 10);
        let cells = piece.cells();
        assert!(cells.contains(&(3, 1)));
        assert!(cells.contains(&(4, 1)));
        assert!(cells.contains(&(5, 1)));
        assert!(cells.contains(&(6, 1)));
    }

    #[test]
    fn rotate_and_back_returns_to_original() {
        let piece = Piece::new(PieceType::T, 10);
        let rotated = piece.rotated(RotateDirection::Cw);
        let back = rotated.rotated(RotateDirection::Ccw);
        assert_eq!(piece.rotation, back.rotation);
    }
}
