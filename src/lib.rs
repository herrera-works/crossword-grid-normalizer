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

    fn is_open(&self, row: usize, col: usize) -> bool {
        self.get(row, col) == Cell::Open
    }

    fn starts_across(&self, row: usize, col: usize) -> bool {
        self.is_open(row, col)
            && !(col > 0 && self.is_open(row, col - 1))
            && col + 1 < self.width
            && self.is_open(row, col + 1)
    }

    fn starts_down(&self, row: usize, col: usize) -> bool {
        self.is_open(row, col)
            && !(row > 0 && self.is_open(row - 1, col))
            && row + 1 < self.height
            && self.is_open(row + 1, col)
    }

    fn across_len(&self, row: usize, col: usize) -> usize {
        (col..self.width)
            .take_while(|&c| self.is_open(row, c))
            .count()
    }

    fn down_len(&self, row: usize, col: usize) -> usize {
        (row..self.height)
            .take_while(|&r| self.is_open(r, col))
            .count()
    }

    /// Returns every pair of squares that breaks 180-degree rotational
    /// symmetry, i.e. where a square and its mirror through the grid's
    /// center disagree. Each pair is reported once, as `(row, col)` of the
    /// earlier square in row-major order followed by its partner, so a
    /// single mismatch is never listed twice. The center square of an
    /// odd-sized grid is its own partner and can't mismatch.
    pub fn symmetry_violations(&self) -> Vec<((usize, usize), (usize, usize))> {
        let mut violations = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                let mirror = (self.height - 1 - row, self.width - 1 - col);
                if (row, col) >= mirror {
                    continue;
                }
                if self.get(row, col) != self.get(mirror.0, mirror.1) {
                    violations.push(((row, col), mirror));
                }
            }
        }
        violations
    }

    /// True if the block pattern looks the same after a half turn, which is
    /// the convention for American-style crosswords.
    pub fn is_rotationally_symmetric(&self) -> bool {
        self.symmetry_violations().is_empty()
    }

    /// Assigns standard crossword numbering: scanning row-major, any open
    /// square that starts an across entry (nothing open to its left, an
    /// open square to its right) and/or a down entry (nothing open above
    /// it, an open square below) gets the next number. A square that starts
    /// both gets one number shared by both its across and down entry, which
    /// is why numbering has to be computed before clues can be listed.
    pub fn number(&self) -> Numbering {
        let mut across = Vec::new();
        let mut down = Vec::new();
        let mut number = 0u32;

        for row in 0..self.height {
            for col in 0..self.width {
                let starts_across = self.starts_across(row, col);
                let starts_down = self.starts_down(row, col);
                if !starts_across && !starts_down {
                    continue;
                }
                number += 1;
                if starts_across {
                    across.push(Clue {
                        number,
                        row,
                        col,
                        len: self.across_len(row, col),
                    });
                }
                if starts_down {
                    down.push(Clue {
                        number,
                        row,
                        col,
                        len: self.down_len(row, col),
                    });
                }
            }
        }

        Numbering { across, down }
    }
}

/// A single across or down entry: the clue number, its starting square, and
/// how many open squares it spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clue {
    pub number: u32,
    pub row: usize,
    pub col: usize,
    pub len: usize,
}

/// The across and down clue numbering derived from a grid's block pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Numbering {
    pub across: Vec<Clue>,
    pub down: Vec<Clue>,
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

    #[test]
    fn numbers_a_simple_grid() {
        // 1 2 3
        // # 4 .
        // 5 . .
        let grid = normalize("...\n#..\n...").unwrap();
        let numbering = grid.number();
        assert_eq!(
            numbering.across,
            vec![
                Clue { number: 1, row: 0, col: 0, len: 3 },
                Clue { number: 4, row: 1, col: 1, len: 2 },
                Clue { number: 5, row: 2, col: 0, len: 3 },
            ]
        );
        assert_eq!(
            numbering.down,
            vec![
                Clue { number: 2, row: 0, col: 1, len: 3 },
                Clue { number: 3, row: 0, col: 2, len: 3 },
            ]
        );
    }

    #[test]
    fn does_not_number_a_single_isolated_square() {
        // The middle square of the bottom row sits between two blocks with
        // nothing below it: it's a dead end, not the start of any entry.
        let grid = normalize("#.#\n...\n#.#").unwrap();
        let numbering = grid.number();
        assert_eq!(numbering.down, vec![Clue { number: 1, row: 0, col: 1, len: 3 }]);
        assert_eq!(numbering.across, vec![Clue { number: 2, row: 1, col: 0, len: 3 }]);
    }

    #[test]
    fn shares_one_number_between_across_and_down_starts() {
        let grid = normalize("..\n..").unwrap();
        let numbering = grid.number();
        assert_eq!(numbering.across[0].number, 1);
        assert_eq!(numbering.down[0].number, 1);
        assert_eq!(numbering.across.len(), 2);
        assert_eq!(numbering.down.len(), 2);
    }

    #[test]
    fn accepts_symmetric_grid() {
        let grid = normalize("#..\n...\n..#").unwrap();
        assert!(grid.is_rotationally_symmetric());
        assert!(grid.symmetry_violations().is_empty());
    }

    #[test]
    fn reports_each_mismatched_pair_once() {
        let grid = normalize("#..\n...\n...").unwrap();
        assert!(!grid.is_rotationally_symmetric());
        assert_eq!(grid.symmetry_violations(), vec![((0, 0), (2, 2))]);
    }

    #[test]
    fn even_sized_grid_symmetry() {
        let ok = normalize("#.\n.#").unwrap();
        assert!(ok.is_rotationally_symmetric());
        let bad = normalize("#.\n##").unwrap();
        assert_eq!(bad.symmetry_violations(), vec![((0, 1), (1, 0))]);
    }

    #[test]
    fn center_square_is_its_own_partner() {
        let grid = normalize("...\n.#.\n...").unwrap();
        assert!(grid.is_rotationally_symmetric());
    }

    #[test]
    fn all_blocked_grid_has_no_clues() {
        let grid = normalize("##\n##").unwrap();
        let numbering = grid.number();
        assert!(numbering.across.is_empty());
        assert!(numbering.down.is_empty());
    }
}
