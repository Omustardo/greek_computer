# Puzzle Generation Plan Summary

## Objective
Add a way to generate games of arbitrary sizes (configurable number of layers, rows, columns, and target sum) within the bounds of the existing `solver.rs`.

## Instructions
1. **Remove Hardcoded Values**: Ensure that values like `NUM_ROWS = 4`, `NUM_COLS = 12`, and the target sum of `42` are removed from the solver and UI, and replaced with dynamically calculated sizes from the puzzle instances.
2. **Connectivity constraints**: When generating the puzzle, all filled cells within a layer must be physically connected to one another (horizontally or vertically) because the layers represent physical concentric disks. Overhangs and multiple holes next to each other are allowed.
3. **Layer appearance constraints**: The topmost rows in the UI (which correspond to the top-most layers) must feature many more holes than the lower layers, enabling the numbers from the bottom layers to peek through.
4. **Unique Solution**: The generation should ideally have only one solution, ensuring playability.
5. **No Python/Metadata Scripts**: Remove any temporary Python or Markdown scripts generated during the planning or scaffolding phases to maintain a clean Rust repository.
6. **UI Integration**: Add simple user controls (e.g., number sliders or inputs) inside `center_panel.rs` to allow the user to input dimensions and spawn a new puzzle seamlessly.

## Implementation Details
- **GameState Updates**: `GameState` and `SessionState` were updated in `app.rs` to track and persist the target sum, row count, column count, and layer count.
- **`puzzle_generator.rs`**: A procedural generation algorithm was created that:
  - Generates a valid answer grid where columns add up to the requested target.
  - Spreads those values across the layers starting from the full complete bottom grid, repeatedly removing removable pieces (cells that do not disconnect the remaining shape when removed).
  - Keeps removing shape pieces layer-by-layer up to the top layer, leaving the top layer with the most holes.
  - Generates decoy values for cells hidden underneath top layers to ensure physical connectivity is maintained without altering the final visible grid.
  - Shuffles the output horizontally so the solution isn't trivial.
