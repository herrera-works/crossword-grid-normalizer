# gridnorm

Crossword grids that get typed up by hand rarely agree on a format. One
person uses `#` for a blocked square, another uses `X`, another pastes in a
solid block character. Open squares show up as `.`, `_`, or `O`. Files
picked up from different editors mix line endings, leave trailing spaces on
some rows, or have a stray blank line between sections. If you're writing a
tool that consumes crossword grids, you end up handling all of that ad hoc,
in every tool, forever.

`gridnorm` parses the common conventions into one canonical `Grid` type and
renders it back out in a single, unambiguous form (`#` for blocked, `.` for
open, one row per line, no trailing whitespace). Every public function is
pure: given the same input it always returns the same output, with no I/O
and no hidden state, so the parsing and formatting logic is easy to unit
test in isolation from anything that reads files or talks to a network.

## What it accepts

- Blocked squares: `#`, `X`, `x`, `*`, or the solid block character `■`
- Open squares: `.`, `_`, `O`, or `o`
- `\n` or `\r\n` line endings
- Trailing whitespace on any row
- Blank lines used as visual separators (ignored)

It rejects rows whose length disagrees with the first row, and any
character it doesn't recognize, rather than silently guessing.

## Library usage

```rust
use gridnorm::{normalize, format_grid};

let messy = "..X \r\n#.* \r\n...  ";
let grid = normalize(messy).expect("valid grid");

assert_eq!(grid.width, 3);
assert_eq!(grid.height, 3);
assert_eq!(format_grid(&grid), "..#\n#.#\n...");
```

## CLI usage

```
gridnorm path/to/grid.txt
```

Given a file containing:

```
...X.
#.*..
.....
..O#.
```

it prints the canonical form to stdout (`X` and `*` become `#`; `O` becomes
`.`, since it's an open-square alias):

```
...#.
#.#..
.....
...#.
```

On invalid input (ragged rows, unrecognized characters, or an empty file)
it prints the reason to stderr and exits non-zero instead of guessing.

## Status

Early skeleton: block/open grids only. No support yet for clue numbers,
symmetry checks, or any file format beyond plain text. See the roadmap in
commit history for what's next.
