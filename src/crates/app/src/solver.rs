use crate::Layer;

/// Represents the solution as a list of rotation amounts for each layer.
/// The `i`-th element is how many times the `i`-th layer needs to be shifted to the right.
pub type Solution = Vec<usize>;

/// Solves the puzzle given the layers and a target sum for all columns.
///
/// Returns `Some(Solution)` if a solution is found, where `Solution` is a list
/// of right-shift amounts for each layer. The base layer (index 0) is never
/// rotated (shift = 0) to eliminate symmetric solutions.
/// Returns `None` if no solution exists.
pub fn solve(layers: &[Layer], target_sum: i32) -> Option<Solution> {
    if layers.is_empty() {
        return Some(vec![]);
    }

    let num_rows = layers[0].values.len();
    if num_rows == 0 {
        return Some(vec![0; layers.len()]);
    }
    let num_cols = layers[0].values[0].len();
    if num_cols == 0 {
        return Some(vec![0; layers.len()]);
    }

    // Validate that all layers have the same dimensions
    for layer in layers {
        assert_eq!(
            layer.values.len(),
            num_rows,
            "All layers must have the same number of rows"
        );
        for row in &layer.values {
            assert_eq!(row.len(), num_cols, "All rows must have the same number of columns");
        }
    }

    let mut current_shifts = vec![0; layers.len()];

    // We maintain a grid of `Option<i16>` to represent the current topmost visible values.
    let mut combined = vec![vec![None; num_cols]; num_rows];
    // Keep track of how many values are visible in each column.
    let mut col_visible_counts = vec![0; num_cols];
    // Keep track of the sum of visible values in each column.
    let mut col_sums = vec![0; num_cols];

    if solve_dfs(
        layers,
        target_sum,
        0,
        &mut current_shifts,
        &mut combined,
        &mut col_visible_counts,
        &mut col_sums,
        num_rows,
        num_cols,
    ) {
        Some(current_shifts)
    } else {
        None
    }
}

fn solve_dfs(
    layers: &[Layer],
    target_sum: i32,
    layer_idx: usize,
    current_shifts: &mut [usize],
    combined: &mut Vec<Vec<Option<i16>>>,
    col_visible_counts: &mut Vec<usize>,
    col_sums: &mut Vec<i32>,
    num_rows: usize,
    num_cols: usize,
) -> bool {
    if layer_idx == layers.len() {
        // Base case: all layers placed.
        // Check if all column sums match the target.
        for sum in col_sums.iter() {
            if *sum != target_sum {
                return false;
            }
        }
        return true;
    }

    // To remove 12-fold rotational symmetry, fix the first layer's rotation to 0.
    let num_shifts = if layer_idx == 0 { 1 } else { num_cols };
    let layer = &layers[layer_idx];

    for shift in 0..num_shifts {
        current_shifts[layer_idx] = shift;

        let mut overwritten = Vec::new();

        for row in 0..num_rows {
            for col in 0..num_cols {
                let orig_col = (col + num_cols - (shift % num_cols)) % num_cols;

                if let Some(val) = layer.values[row][orig_col] {
                    if combined[row][col].is_none() {
                        combined[row][col] = Some(val);
                        col_sums[col] += i32::from(val);
                        col_visible_counts[col] += 1;
                        overwritten.push((row, col, val));
                    }
                }
            }
        }

        // Pruning: check if any column is FULLY filled and its sum is NOT target_sum.
        // Alternatively, if a column is NOT fully filled but its sum exceeds target_sum
        // (assuming all numbers are positive), we could also prune.
        // For the Greek Computer, all numbers are non-negative, but let's just prune
        // fully filled columns for generality, unless we know they're positive.
        // Actually, looking at the default puzzle, all numbers are >= 0.
        // We'll just prune if a column is full and the sum is wrong.
        let mut possible = true;
        for col in 0..num_cols {
            if col_visible_counts[col] == num_rows && col_sums[col] != target_sum {
                possible = false;
                break;
            }
        }

        if possible {
            if solve_dfs(
                layers,
                target_sum,
                layer_idx + 1,
                current_shifts,
                combined,
                col_visible_counts,
                col_sums,
                num_rows,
                num_cols,
            ) {
                return true;
            }
        }

        // Backtrack
        for (row, col, val) in overwritten {
            combined[row][col] = None;
            col_sums[col] -= i32::from(val);
            col_visible_counts[col] -= 1;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_small_board() {
        let layer1 = Layer::new_from_text(2, 2, "1 __\n2 3");
        let layer2 = Layer::new_from_text(2, 2, "4 5\n__ 6");
        let layers = vec![layer1, layer2];
        let target_sum = 6;

        let solution = solve(&layers, target_sum);
        // layer1:
        // [1, None]
        // [2, 3]
        // layer2:
        // [4, 5]
        // [None, 6]

        // If shifts=[0, 0]:
        // Combined:
        // [1, 5]
        // [2, 3]
        // Col 0 sum: 1 + 2 = 3
        // Col 1 sum: 5 + 3 = 8
        // Not target sum (6).

        // If shifts=[0, 1] (layer2 right shifted by 1):
        // layer2 shifted:
        // [5, 4]
        // [6, None]
        // Combined with layer1:
        // [1, 4]
        // [2, 3]
        // Wait, layer2 shifted:
        // row 0: [5, 4]
        // row 1: [6, None]
        // combined row 0: [1, 4]
        // combined row 1: [2, 3]
        // Col 0 sum: 1 + 2 = 3
        // Col 1 sum: 4 + 3 = 7
        // Not target sum.

        // Let's modify the target_sum to whatever is solvable, or change the board.
        // Actually we just want a test that doesn't panic and proves it can return None if no solution.
        let solution = solve(&layers, 100);
        assert!(solution.is_none());
    }

    #[test]
    fn test_solver_default_layers() {
        let layers = Layer::default_layers();
        let target_sum = 42;

        let solution = solve(&layers, target_sum);
        assert!(solution.is_some(), "Solver should find a solution for default layers");

        if let Some(shifts) = solution {
            println!("Found shifts: {:?}", shifts);

            // Verify solution
            let num_rows = layers[0].values.len();
            let num_cols = layers[0].values[0].len();

            let mut combined = vec![vec![None; num_cols]; num_rows];
            let mut col_sums = vec![0; num_cols];

            for (i, layer) in layers.iter().enumerate() {
                let shift = shifts[i];
                for row in 0..num_rows {
                    for col in 0..num_cols {
                        let orig_col = (col + num_cols - (shift % num_cols)) % num_cols;
                        if let Some(val) = layer.values[row][orig_col] {
                            if combined[row][col].is_none() {
                                combined[row][col] = Some(val);
                                col_sums[col] += i32::from(val);
                            }
                        }
                    }
                }
            }

            for sum in col_sums {
                assert_eq!(sum, target_sum, "Column sum must match target");
            }
        }
    }
}
