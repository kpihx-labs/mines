# mines

`mines` is a fast, responsive Minesweeper built natively in Rust for real terminal use: a tiny split pane, a standard shell, tmux, SSH, or a large fullscreen terminal.

## Why

Classic console Minesweeper implementations tend to assume an 80-column terminal, render tiny fixed cells, or expose mouse input at the expense of keyboard control. `mines` treats terminal geometry as live input. It expands cells when space exists and creates a cursor-centered viewport when the complete field cannot fit.

## Features

- Beginner `9x9/10`, intermediate `16x16/40`, and expert `30x16/99` presets.
- Arbitrary custom fields from `5x5` through `100x100`.
- A guaranteed-safe 3x3 first-click zone.
- Iterative flood fill with no recursion-depth risk.
- Chording on revealed numbers.
- Keyboard and mouse input with terminal state restored on every normal exit.
- Responsive cells and automatic panning around the cursor.
- Frozen win/loss board with explicit replay, scores, and quit actions.
- Persistent top-ten times per exact field configuration.
- Deterministic seeds for reproducible boards and debugging.

## Install

```bash
make install
```

The binary is installed through Cargo as `mines`.

## Play

```bash
mines
mines --difficulty beginner
mines --difficulty expert
mines --difficulty custom --width 40 --height 20 --mines 130
mines --difficulty expert --seed 42
```

| Action | Keyboard | Mouse |
|---|---|---|
| Move | Arrows or `hjkl` | Move pointer over a cell |
| Reveal / chord | `Space` or `Enter` | Left click |
| Flag / unflag | `F` | Right click |
| Best times | `S` | — |
| Help | `?` | — |
| Restart | `R` | — |
| Quit / close overlay | `Q` or `Esc` | — |

## Persistence

Scores live under the platform data directory at `mines/scores.json`, normally `$HOME/.local/share/mines/scores.json` on Linux. Only completed fields are ranked.

## References

The ignored `refs/` workspace contains read-only research clones:

- `rwestlund/freesweep`: historical C/ncurses behavior and large-board precedent.
- `jesper-olsen/minesweeper-rs`: first-click policy and solver-oriented CLI research.
- `meihapps/minesweepe-rs`: modern no-guess, hint, mouse, and leaderboard research.

No source code from those projects is copied into `mines`; this implementation is original.

## Quality gate

```bash
make check
```

This runs formatting, Clippy with warnings denied, all tests, a locked release build, and CLI smoke tests.

## License

Business Source License 1.1. Personal, noncommercial, teaching, and academic production use is granted; the project converts to `AGPL-3.0-only` on 2030-11-05.
