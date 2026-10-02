//! Board geometry, indexing, and neighbor utilities for the chain-reaction grid.

use crate::{cell::Cell, error::InvalidBoard, player::PlayerId};
use serde::{Deserialize, Serialize};

/// Flat index type addressing cells in the board's internal vector.
pub type CellIndex = u32;

/// Flat grid storage for all cells in row-major order.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Board {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

/// Cardinal directions used for orthogonal neighbor traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    /// Deterministic UP→RIGHT→DOWN→LEFT ordering used throughout the engine.
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];
}

impl Board {
    /// Creates an empty board of the requested dimensions, rejecting zero-sized grids.
    pub fn new(width: u16, height: u16) -> Result<Self, InvalidBoard> {
        if width == 0 || height == 0 {
            return Err(InvalidBoard::ZeroDimension);
        }
        if width < 2 || height < 2 {
            return Err(InvalidBoard::TooSmall);
        }
        let cell_count = width as usize * height as usize;
        Ok(Self {
            width,
            height,
            cells: vec![Cell::empty(); cell_count],
        })
    }

    /// Returns the board width in cells.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Returns the board height in cells.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns the total number of cells on the board.
    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns true if the board has zero cells.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Converts a (row, col) pair into a flat cell index, returning None if out of bounds.
    #[inline]
    pub fn index(&self, row: u16, col: u16) -> Option<usize> {
        if row < self.height && col < self.width {
            Some((row as usize) * self.width as usize + col as usize)
        } else {
            None
        }
    }

    /// Converts a flat cell index back into (row, col) coordinates.
    #[inline]
    pub fn coords(&self, index: usize) -> (u16, u16) {
        let row = index / self.width as usize;
        let col = index % self.width as usize;
        (row as u16, col as u16)
    }

    /// Returns an immutable reference to the cell at the given index.
    #[inline]
    pub fn cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    /// Returns a mutable reference to the cell at the given index.
    #[inline]
    pub fn cell_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.cells[index]
    }

    /// Looks up the critical mass for a cell identified by flat index.
    pub fn critical_mass_index(&self, index: usize) -> u32 {
        let (row, col) = self.coords(index);
        self.critical_mass(row, col)
    }

    /// Returns the critical mass for the cell at the given coordinates based on neighbors.
    pub fn critical_mass(&self, row: u16, col: u16) -> u32 {
        let mut count = 0;
        if row > 0 {
            count += 1;
        }
        if row + 1 < self.height {
            count += 1;
        }
        if col > 0 {
            count += 1;
        }
        if col + 1 < self.width {
            count += 1;
        }
        count
    }

    /// Iterates over neighbor indices in deterministic UP→RIGHT→DOWN→LEFT order.
    pub fn neighbors(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        Direction::ALL
            .into_iter()
            .filter_map(move |direction| self.neighbor(index, direction))
    }

    /// Returns the neighbor index in a given direction, or None if the edge blocks it.
    pub fn neighbor(&self, index: usize, direction: Direction) -> Option<usize> {
        let (row, col) = self.coords(index);
        match direction {
            Direction::Up => {
                if row == 0 {
                    None
                } else {
                    self.index(row - 1, col)
                }
            }
            Direction::Right => {
                if col + 1 >= self.width {
                    None
                } else {
                    self.index(row, col + 1)
                }
            }
            Direction::Down => {
                if row + 1 >= self.height {
                    None
                } else {
                    self.index(row + 1, col)
                }
            }
            Direction::Left => {
                if col == 0 {
                    None
                } else {
                    self.index(row, col - 1)
                }
            }
        }
    }

    /// Counts how many non-empty cells currently belong to the given player.
    pub fn owner_cell_count(&self, player: PlayerId) -> usize {
        self.cells
            .iter()
            .filter(|cell| cell.owner == Some(player) && cell.orb_count > 0)
            .count()
    }
}
