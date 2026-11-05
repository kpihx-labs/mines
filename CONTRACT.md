# Runtime Contract

## Engine

- The first reveal generates exactly the configured mine count.
- The first cell and every in-bounds neighbor within one row and column are mine-free.
- Revealing a zero performs iterative flood fill across connected safe cells.
- Revealing a numbered cell with the matching adjacent flag count chords its remaining neighbors.
- A win occurs only when every non-mine cell is revealed.
- A loss reveals every mine and freezes the completed board.

## Interface

- The complete field is centered whenever it fits.
- When it cannot fit, a cursor-centered viewport exposes the maximum playable area.
- Keyboard support remains complete without a mouse.
- Mouse capture is released and terminal state restored on exit.
- Resize events are handled by redrawing from current terminal geometry.

## Persistence

- Only won games are ranked.
- Rankings are scoped to exact difficulty name, dimensions, and mine count.
- The ten fastest matching results are retained and displayed independently for every exact field configuration.
- Data uses the platform data directory, never a hardcoded home path.
