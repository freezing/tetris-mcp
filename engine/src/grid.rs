use serde::de::Deserializer;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use crate::piece::PieceType;

/// A sparse cell entry: (x, y, piece_type).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SparseCell {
    x: usize,
    y: usize,
    piece: PieceType,
}

/// A 2D grid of cells, each optionally containing a piece type.
///
/// This is a pure data structure — it handles storage, bounds checking,
/// and row-level operations. It has no knowledge of game rules, pieces,
/// or movement. Game logic lives in `Board` (piece locking, line clearing)
/// and `Game` (turns, scoring).
///
/// Serializes sparsely: only non-empty cells are stored in JSON.
#[derive(Debug, Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Option<PieceType>>,
}

impl Serialize for Grid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let occupied: Vec<SparseCell> = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| {
                cell.map(|piece| SparseCell {
                    x: i % self.width,
                    y: i / self.width,
                    piece,
                })
            })
            .collect();

        let mut state = serializer.serialize_struct("Grid", 3)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("cells", &occupied)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Grid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct GridData {
            width: usize,
            height: usize,
            cells: Vec<SparseCell>,
        }

        let data = GridData::deserialize(deserializer)?;
        let mut grid = Grid::new(data.width, data.height);
        for cell in data.cells {
            if grid.in_bounds(cell.x as i32, cell.y as i32) {
                grid.set(cell.x, cell.y, Some(cell.piece));
            }
        }
        Ok(grid)
    }
}

impl Grid {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<PieceType> {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: Option<PieceType>) {
        self.cells[y * self.width + x] = value;
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.get(x as usize, y as usize).is_none()
    }

    pub fn row_full(&self, y: usize) -> bool {
        (0..self.width).all(|x| self.get(x, y).is_some())
    }

    pub fn clear_row(&mut self, y: usize) {
        for x in 0..self.width {
            self.set(x, y, None);
        }
    }

    pub fn copy_row(&mut self, from_y: usize, to_y: usize) {
        for x in 0..self.width {
            let val = self.get(x, from_y);
            self.set(x, to_y, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_empty() {
        let grid = Grid::new(10, 20);
        for y in 0..20 {
            for x in 0..10 {
                assert!(grid.get(x, y).is_none());
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut grid = Grid::new(10, 20);
        grid.set(3, 5, Some(PieceType::T));
        assert_eq!(grid.get(3, 5), Some(PieceType::T));
        assert_eq!(grid.get(4, 5), None);
    }

    #[test]
    fn bounds_check() {
        let grid = Grid::new(10, 20);
        assert!(grid.in_bounds(0, 0));
        assert!(grid.in_bounds(9, 19));
        assert!(!grid.in_bounds(-1, 0));
        assert!(!grid.in_bounds(10, 0));
        assert!(!grid.in_bounds(0, 20));
    }

    #[test]
    fn row_full_detection() {
        let mut grid = Grid::new(10, 20);
        assert!(!grid.row_full(19));

        for x in 0..10 {
            grid.set(x, 19, Some(PieceType::I));
        }
        assert!(grid.row_full(19));
    }

    #[test]
    fn copy_row_works() {
        let mut grid = Grid::new(10, 20);
        grid.set(0, 5, Some(PieceType::T));
        grid.set(3, 5, Some(PieceType::J));

        grid.copy_row(5, 10);
        assert_eq!(grid.get(0, 10), Some(PieceType::T));
        assert_eq!(grid.get(3, 10), Some(PieceType::J));
        assert_eq!(grid.get(1, 10), None);
    }

    #[test]
    fn empty_grid_serializes_sparse() {
        let grid = Grid::new(10, 20);
        let json = serde_json::to_string(&grid).unwrap();
        // Empty grid should have no cell entries
        assert!(json.contains("\"cells\":[]"), "Expected empty cells array, got: {json}");
    }

    #[test]
    fn sparse_roundtrip() {
        let mut grid = Grid::new(10, 20);
        grid.set(3, 5, Some(PieceType::T));
        grid.set(7, 19, Some(PieceType::I));

        let json = serde_json::to_string(&grid).unwrap();
        let restored: Grid = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.width(), 10);
        assert_eq!(restored.height(), 20);
        assert_eq!(restored.get(3, 5), Some(PieceType::T));
        assert_eq!(restored.get(7, 19), Some(PieceType::I));
        assert_eq!(restored.get(0, 0), None);
    }

    #[test]
    fn sparse_json_is_small() {
        let grid = Grid::new(10, 20);
        let json = serde_json::to_string(&grid).unwrap();
        // Empty 10x20 grid should be tiny — just width, height, empty array
        assert!(json.len() < 50, "Empty grid JSON too large: {} bytes", json.len());
    }
}
