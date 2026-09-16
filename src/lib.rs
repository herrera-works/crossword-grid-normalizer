//! Normalizes messy crossword grid text into a canonical form.
//!
//! People write crossword grids by hand in all sorts of ways: '#', 'X', or
//! a solid block character for blocked squares, '.', '_', or 'O' for open
//! ones, inconsistent line endings, trailing spaces, stray blank lines. This
//! crate turns that mess into one canonical `Grid` so downstream code never
//! has to guess which convention a particular file used.

use std::fmt;

/// A single square in a crossword grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Block,
    Open,
}

/// A rectangular crossword grid, stored row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Builds a grid from a flat, row-major cell list. Panics if the cell
    /// count doesn't match `width * height`, since that would mean the
    /// caller assembled the grid incorrectly.
    pub fn new(width: usize, height: usize, cells: Vec<Cell>) -> Self {
        assert_eq!(
            cells.len(),
            width * height,
            "cell count must match width * height"
        );
        Grid {
            width,
            height,
            cells,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        self.cells[row * self.width + col]
    }

    pub fn row(&self, row: usize) -> &[Cell] {
        let start = row * self.width;
        &self.cells[start..start + self.width]
    }
}

/// Why a piece of input couldn't be turned into a `Grid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeError {
    EmptyInput,
    RaggedRow {
        row: usize,
        expected: usize,
        found: usize,
    },
    UnknownChar {
        row: usize,
        col: usize,
        ch: char,
    },
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalizeError::EmptyInput => write!(f, "input has no grid rows"),
            NormalizeError::RaggedRow {
                row,
                expected,
                found,
            } => write!(f, "row {} has {} columns, expected {}", row, found, expected),
            NormalizeError::UnknownChar { row, col, ch } => write!(
                f,
                "unrecognized character {:?} at row {}, col {}",
                ch, row, col
            ),
        }
    }
}

impl std::error::Error for NormalizeError {}

/// Splits raw input into candidate grid rows: strips a trailing '\r',
/// trims trailing whitespace, and drops lines that are blank once trimmed.
pub fn significant_lines(input: &str) -> Vec<&str> {
    input
        .lines()
        .map(|line| line.trim_end_matches('\r').trim_end())
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// Maps a single character to a cell, or `None` if it isn't recognized.
pub fn classify_char(ch: char) -> Option<Cell> {
    match ch {
        '#' | 'X' | 'x' | '*' | '\u{25A0}' => Some(Cell::Block),
        '.' | '_' | 'O' | 'o' => Some(Cell::Open),
        _ => None,
    }
}

/// Parses messy crossword grid text into a canonical `Grid`.
///
/// Tolerates a handful of common conventions for blocked squares (`#`, `X`,
/// `x`, `*`, the solid block character) and open squares (`.`, `_`, `O`,
/// `o`), CRLF line endings, trailing whitespace, and blank separator lines.
/// Rejects any row whose column count disagrees with the first row, and any
/// character it doesn't recognize.
pub fn normalize(input: &str) -> Result<Grid, NormalizeError> {
    let lines = significant_lines(input);
    if lines.is_empty() {
        return Err(NormalizeError::EmptyInput);
    }

    let width = lines[0].chars().count();
    let mut cells = Vec::with_capacity(width * lines.len());

    for (row, line) in lines.iter().enumerate() {
        let found = line.chars().count();
        if found != width {
            return Err(NormalizeError::RaggedRow {
                row,
                expected: width,
                found,
            });
        }
        for (col, ch) in line.chars().enumerate() {
            match classify_char(ch) {
                Some(cell) => cells.push(cell),
                None => return Err(NormalizeError::UnknownChar { row, col, ch }),
            }
        }
    }

    Ok(Grid::new(width, lines.len(), cells))
}

/// Renders a grid back into its canonical text form: `#` for blocked
/// squares, `.` for open squares, one row per line, no trailing whitespace.
pub fn format_grid(grid: &Grid) -> String {
    let mut out = String::with_capacity(grid.width * grid.height + grid.height);
    for row in 0..grid.height {
        for cell in grid.row(row) {
            out.push(match cell {
                Cell::Block => '#',
                Cell::Open => '.',
            });
        }
        if row + 1 < grid.height {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mixed_symbols() {
        let input = "..X\n#.*\n...";
        let grid = normalize(input).unwrap();
        assert_eq!(format_grid(&grid), "..#\n#.#\n...");
    }

    #[test]
    fn strips_trailing_whitespace_and_crlf() {
        let input = "...  \r\n#.#\r\n...\r\n";
        let grid = normalize(input).unwrap();
        assert_eq!(grid.width, 3);
        assert_eq!(grid.height, 3);
    }

    #[test]
    fn ignores_blank_separator_lines() {
        let input = "...\n\n#.#\n\n...";
        let grid = normalize(input).unwrap();
        assert_eq!(grid.height, 3);
    }

    #[test]
    fn rejects_ragged_rows() {
        let input = "...\n#.\n...";
        let err = normalize(input).unwrap_err();
        assert_eq!(
            err,
            NormalizeError::RaggedRow {
                row: 1,
                expected: 3,
                found: 2
            }
        );
    }

    #[test]
    fn rejects_unknown_characters() {
        let input = "..?\n#.#\n...";
        let err = normalize(input).unwrap_err();
        assert_eq!(
            err,
            NormalizeError::UnknownChar {
                row: 0,
                col: 2,
                ch: '?'
            }
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(normalize("").unwrap_err(), NormalizeError::EmptyInput);
        assert_eq!(normalize("   \n\n").unwrap_err(), NormalizeError::EmptyInput);
    }

    #[test]
    fn round_trips_through_canonical_form() {
        let input = "..#\n#.#\n...";
        let grid = normalize(input).unwrap();
        let canonical = format_grid(&grid);
        let reparsed = normalize(&canonical).unwrap();
        assert_eq!(grid, reparsed);
    }
}
