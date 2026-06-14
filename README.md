# sudoku-rust

![Solver animation](./asset/trim.gif)

A 9×9 Sudoku solver written in Rust as a learning project. The focus is on Rust
idioms (ownership, borrowing, threads, channels) while building working solvers
and a small CLI that visualizes the board as it is filled in.

Three algorithms are included:

| Algorithm | CLI flag | Notes |
|-----------|----------|-------|
| **Candidate election** (default) | `ce`, `candidate` | Backtracking with precomputed candidates per cell |
| **Backtracking** | `bt`, `backtracking` | Straightforward try-and-backtrack |
| **Simulated annealing** | `sa`, `simulatedannealing` | Stochastic search based on [Lewis (2007)](https://rhydlewis.eu/papers/META_CAN_SOLVE_SUDOKU.pdf); fast on typical puzzles, may not finish hard ones |

After each run, perf stats are printed to stderr:

```
Perf: actions=118849 elapsed=0.034960s
```

## Input format

Provide a 9-line text file (default: `input.txt`) where each line has 9
characters:

- Digits `1`–`9` — fixed (given) cells
- Any non-digit (commonly `?` or `.`) — empty cell

Example (`example/expert.txt`):

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

Example puzzles in `example/`:

| File | Difficulty |
|------|------------|
| `easy.txt` | Easy |
| `wikipedia.txt` | Medium (classic Wikipedia example) |
| `expert.txt` | Hard |

Copy one to `input.txt` before running, e.g. `cp example/wikipedia.txt input.txt`.

## Build & run

```bash
cargo build --release
cp example/wikipedia.txt input.txt
cargo run --release
```

### CLI options

- `--throttle-ms <ms>` — delay between live board updates (default: 100). Only
  affects backtracking and candidate election; simulated annealing updates the
  board once at the end.
- `--algorithm <name>` / `-a <name>` — solver to use (see table above).

Examples:

```bash
# Candidate election with faster UI updates
cargo run --release -- --throttle-ms 50

# Backtracking
cargo run --release -- -a bt

# Simulated annealing
cargo run --release -- -a sa
```

The terminal prints an ANSI-coloured 9×9 grid: fixed digits in **blue**,
solver-filled digits in **yellow**, empty cells as **?** in red.

## References

- Sudoku solving algorithms — [Wikipedia](https://en.wikipedia.org/wiki/Sudoku_solving_algorithms)
- Simulated annealing for Sudoku — [Lewis (2007)](https://rhydlewis.eu/papers/META_CAN_SOLVE_SUDOKU.pdf)
- [NYT Sudoku](https://www.nytimes.com/puzzles/sudoku)

## License / notes

This repository is for learning and experimentation. You're free to use the
code for personal projects and study.
