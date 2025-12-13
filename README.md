# sudoku-rust

![Backtrack Solution](./asset/trim.gif)

A small Sudoku solver written in Rust as a learning project. The goal of this
repository is to practice Rust idioms (ownership, borrowing, threads, channels)
while implementing a working Sudoku solver and a tiny CLI-based UI.

The solver uses a backtracking algorithm and a small performance tracker to
report how many actions were attempted and the elapsed time.

## Input format

Provide a 9-line text file (default: `input.txt`) where each line has 9
characters. Digits `1`..`9` represent fixed cell values. Any non-digit
character (commonly `?`) is treated as an empty cell. Example:

```
??173???2
5??6?????
?3???8???
?26?????4
???4???2?
4????59?8
?5??16???
2??3??1?6
??9??????
```

There are example puzzles in the `example/` folder.
## Build & run

This project uses Cargo. From the repository root:

```bash
cargo build --release
```

To run the solver using the default `input.txt` file:

```bash
cargo run --release
```

CLI options:

- `--throttle` — enable throttling of board print updates (useful for slow
	machines or to visually follow the solver).
- `--throttle-ms <ms>` — set throttle interval in milliseconds (also enables
	throttling). Defaults to 100ms when enabled.

Example with throttle set to 50ms:

```bash
cargo run --release -- --throttle-ms 50
```

Notes: the project prints an ANSI-coloured board. Fixed (given) digits are
printed in blue, solver-filled digits in yellow, and unknown cells in red.


## References

- Sudoku solving algorithms — Wikipedia: https://en.wikipedia.org/wiki/Sudoku_solving_algorithms
- Play Sudoku (NYT): https://www.nytimes.com/puzzles/sudoku

## License / Notes

This repository was created for learning and experimentation. You're free to
use the code for personal projects and study.

