use crate::Layer;
use rand::seq::IndexedRandom;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn generate_puzzle(
    num_layers: usize,
    num_rows: usize,
    num_cols: usize,
    target_sum: i32,
) -> Vec<Layer> {
    if num_layers == 0 || num_rows == 0 || num_cols == 0 {
        return vec![];
    }

    let mut rng = rand::rng();

    // We want to generate a puzzle that has exactly 1 solution if possible.
    // We'll generate a few and pick the one with the fewest solutions.
    // However, `solve` currently just returns `Option<Solution>`. We might need to
    // just generate one and hope for the best, or we can copy/modify the solver
    // to count solutions if unique solution is strictly required. For now, we just
    // generate one good puzzle.

    // Step 1: Create a valid solved grid.
    // Each column must sum to `target_sum`.
    let mut solved_grid = vec![vec![0i16; num_cols]; num_rows];
    for col in 0..num_cols {
        let mut remaining = target_sum;
        for row in 0..num_rows - 1 {
            // Pick a random value. If remaining is 0, it has to be 0.
            // To ensure non-negative values, we pick between 0 and remaining.
            let val = if remaining > 0 {
                rng.random_range(0..=remaining)
            } else {
                0
            };
            solved_grid[row][col] = val as i16;
            remaining -= val;
        }
        solved_grid[num_rows - 1][col] = remaining as i16;

        // Shuffle the column so the bottom row isn't biased to have the remainder.
        let mut col_vals: Vec<i16> = (0..num_rows).map(|r| solved_grid[r][col]).collect();
        col_vals.shuffle(&mut rng);
        for row in 0..num_rows {
            solved_grid[row][col] = col_vals[row];
        }
    }

    // Step 2: Assign each cell (r, c) exactly one visible layer.
    // To ensure physical possibility (connectivity) and that the top layer has many holes,
    // we use the strategy:
    // S_{num_layers-1} = full grid.
    // S_l is formed by removing some removable cells from S_{l+1}.
    // A cell is removable if its removal doesn't disconnect the set S_{l+1} (wrapping around horizontally).

    let mut current_set = vec![vec![true; num_cols]; num_rows];
    let mut visible_layer = vec![vec![num_layers - 1; num_cols]; num_rows];

    for l in (0..num_layers - 1).rev() {
        // We want to remove a fraction of the current cells.
        // Let's say we want the sizes of sets to grow roughly linearly or exponentially.
        // Actually, just randomly removing a bunch of removable cells is fine.
        let target_size = (num_rows * num_cols) * (l + 1) / num_layers;
        let mut current_size = current_set.iter().flat_map(|r| r.iter()).filter(|&&b| b).count();

        while current_size > target_size {
            let removable = get_removable_cells(&current_set);
            if removable.is_empty() {
                break; // Cannot remove any more cells without disconnecting
            }
            let &(r, c) = removable.choose(&mut rng).unwrap();
            current_set[r][c] = false;
            // The cell removed was in S_{l+1} but not S_l.
            // This means its visible layer is exactly l + 1.
            visible_layer[r][c] = l + 1;
            current_size -= 1;
        }

        // Cells still in current_set will have visible_layer <= l.
        for r in 0..num_rows {
            for c in 0..num_cols {
                if current_set[r][c] {
                    visible_layer[r][c] = l;
                }
            }
        }
    }

    // Step 3: Populate the layers
    let mut layers = Vec::with_capacity(num_layers);
    for l in 0..num_layers {
        let mut values = vec![vec![None; num_cols]; num_rows];
        for r in 0..num_rows {
            for c in 0..num_cols {
                let vis = visible_layer[r][c];
                if vis < l {
                    // This cell is hidden by an upper layer.
                    // To ensure connectivity, we fill it with a decoy number,
                    // but ONLY if the cell is in S_l. Wait, if it's in S_l, it has vis <= l.
                    // If vis < l, it is in S_l, so we MUST fill it.
                    values[r][c] = Some(rng.random_range(0..=target_sum) as i16);
                } else if vis == l {
                    // This cell is visible on this layer.
                    values[r][c] = Some(solved_grid[r][c]);
                } else {
                    // vis > l. This cell must be a hole to let lower layers show through.
                    values[r][c] = None;
                }
            }
        }
        layers.push(Layer { values });
    }

    // Step 4: Scramble the layers
    for (l, layer) in layers.iter_mut().enumerate() {
        if l == 0 {
            // Keep top layer unshifted to prevent symmetric identical puzzles
            continue;
        }
        let shift = rng.random_range(0..num_cols);
        for row in &mut layer.values {
            row.rotate_right(shift);
        }
    }

    layers
}

// Helper to find cells that can be removed without disconnecting the true values in `grid`
// `grid` wraps horizontally but not vertically.
fn get_removable_cells(grid: &[Vec<bool>]) -> Vec<(usize, usize)> {
    let num_rows = grid.len();
    let num_cols = grid[0].len();
    let mut removable = Vec::new();

    let total_true = grid.iter().flat_map(|r| r.iter()).filter(|&&b| b).count();
    if total_true <= 1 {
        return vec![];
    }

    for r in 0..num_rows {
        for c in 0..num_cols {
            if grid[r][c] {
                // Check if removing (r, c) disconnects the remaining true cells.
                let mut temp_grid = grid.to_vec();
                temp_grid[r][c] = false;
                if is_connected(&temp_grid, total_true - 1) {
                    removable.push((r, c));
                }
            }
        }
    }
    removable
}

fn is_connected(grid: &[Vec<bool>], expected_count: usize) -> bool {
    if expected_count == 0 {
        return true;
    }
    let num_rows = grid.len();
    let num_cols = grid[0].len();

    let mut start = None;
    for r in 0..num_rows {
        for c in 0..num_cols {
            if grid[r][c] {
                start = Some((r, c));
                break;
            }
        }
        if start.is_some() {
            break;
        }
    }

    let start = match start {
        Some(s) => s,
        None => return true,
    };

    let mut visited = vec![vec![false; num_cols]; num_rows];
    let mut stack = vec![start];
    visited[start.0][start.1] = true;
    let mut count = 1;

    while let Some((r, c)) = stack.pop() {
        let mut neighbors = vec![];
        if r > 0 { neighbors.push((r - 1, c)); }
        if r < num_rows - 1 { neighbors.push((r + 1, c)); }
        neighbors.push((r, (c + num_cols - 1) % num_cols));
        neighbors.push((r, (c + 1) % num_cols));

        for (nr, nc) in neighbors {
            if grid[nr][nc] && !visited[nr][nc] {
                visited[nr][nc] = true;
                count += 1;
                stack.push((nr, nc));
            }
        }
    }

    count == expected_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::solve;

    #[test]
    fn test_generator_valid() {
        let layers = generate_puzzle(5, 4, 12, 42);
        assert_eq!(layers.len(), 5);
        assert_eq!(layers[0].values.len(), 4);
        assert_eq!(layers[0].values[0].len(), 12);

        let solution = solve(&layers, 42);
        assert!(solution.is_some(), "Generated puzzle must be solvable");
    }
}
