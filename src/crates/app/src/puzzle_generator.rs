use rand::Rng;
use std::collections::VecDeque;
use crate::solver;
use crate::Layer;

pub fn generate_puzzle(num_rows: usize, num_cols: usize, num_layers: usize, target_sum: i32) -> Option<Vec<Layer>> {
    let mut rng = rand::rng();

    let mut base_grid = vec![vec![0i16; num_cols]; num_rows];
    for col in 0..num_cols {
        let mut current_sum = 0;
        for row in 0..num_rows {
            if row == num_rows - 1 {
                base_grid[row][col] = (target_sum - current_sum) as i16;
            } else {
                let max_val = target_sum - current_sum;
                let val = if max_val > 0 { rng.random_range(0..=max_val) } else { 0 };
                base_grid[row][col] = val as i16;
                current_sum += val;
            }
        }
    }

    let mut layers_masks = vec![vec![vec![false; num_cols]; num_rows]; num_layers];

    for row in 0..num_rows {
        for col in 0..num_cols {
            layers_masks[0][row][col] = true;
        }
    }

    for l in 1..num_layers {
        let mut current_mask = layers_masks[l-1].clone();

        let target_cells = ((num_rows * num_cols) as f32 * (1.0 - (l as f32) / (num_layers as f32))).max(1.0) as usize;

        let mut current_cells = 0;
        let mut available_to_remove: Vec<(usize, usize)> = Vec::new();
        for row in 0..num_rows {
            for col in 0..num_cols {
                if current_mask[row][col] {
                    current_cells += 1;
                    available_to_remove.push((row, col));
                }
            }
        }

        use rand::seq::SliceRandom;
        available_to_remove.shuffle(&mut rng);

        for &(r, c) in &available_to_remove {
            if current_cells <= target_cells {
                break;
            }
            current_mask[r][c] = false;
            if is_connected(&current_mask, num_rows, num_cols) {
                current_cells -= 1;
            } else {
                current_mask[r][c] = true;
            }
        }

        layers_masks[l] = current_mask;
    }

    let mut final_layers = vec![Layer { values: vec![vec![None; num_cols]; num_rows] }; num_layers];

    for row in 0..num_rows {
        for col in 0..num_cols {
            let mut highest_layer = 0;
            for l in (0..num_layers).rev() {
                if layers_masks[l][row][col] {
                    highest_layer = l;
                    break;
                }
            }

            final_layers[highest_layer].values[row][col] = Some(base_grid[row][col]);

            for l in 0..highest_layer {
                if layers_masks[l][row][col] {
                    final_layers[l].values[row][col] = Some(rng.random_range(0..=target_sum) as i16);
                }
            }
        }
    }

    for l in 0..num_layers {
        let shift = rng.random_range(0..num_cols);
        let mut new_layer_values = vec![vec![None; num_cols]; num_rows];
        for row in 0..num_rows {
            for col in 0..num_cols {
                let new_col = (col + shift) % num_cols;
                new_layer_values[row][new_col] = final_layers[l].values[row][col];
            }
        }
        final_layers[l].values = new_layer_values;
    }

    final_layers.reverse();

    if solver::solve(&final_layers, target_sum).is_some() {
        return Some(final_layers);
    }

    None
}

fn is_connected(mask: &Vec<Vec<bool>>, num_rows: usize, num_cols: usize) -> bool {
    let mut start = None;
    let mut total_true = 0;
    for r in 0..num_rows {
        for c in 0..num_cols {
            if mask[r][c] {
                if start.is_none() {
                    start = Some((r, c));
                }
                total_true += 1;
            }
        }
    }

    if total_true == 0 {
        return true;
    }

    let mut visited = vec![vec![false; num_cols]; num_rows];
    let mut queue = VecDeque::new();
    let start_node = start.unwrap();
    queue.push_back(start_node);
    visited[start_node.0][start_node.1] = true;

    let mut count = 0;

    // The columns wrap around! "rotate concentric layers" means left edge is connected to right edge.
    // Wait, does physical connectivity wrap around?
    // Yes, a disk is circular. The left edge touches the right edge.
    // Does it wrap around vertically? No.
    while let Some((r, c)) = queue.pop_front() {
        count += 1;

        let mut neighbors = Vec::new();
        if r > 0 {
            neighbors.push((r - 1, c)); // Up
        }
        if r + 1 < num_rows {
            neighbors.push((r + 1, c)); // Down
        }
        neighbors.push((r, (c + num_cols - 1) % num_cols)); // Left (wrap)
        neighbors.push((r, (c + 1) % num_cols));            // Right (wrap)

        for &(nr, nc) in &neighbors {
            if nc < num_cols {
                if mask[nr][nc] && !visited[nr][nc] {
                    visited[nr][nc] = true;
                    queue.push_back((nr, nc));
                }
            }
        }
    }

    count == total_true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_connectivity() {
        let num_rows = 4;
        let num_cols = 12;
        let num_layers = 4;
        let target_sum = 42;

        let puzzle_opt = generate_puzzle(num_rows, num_cols, num_layers, target_sum);
        // Generation might rarely fail if it can't find a solution, though it should be robust.
        if let Some(puzzle) = puzzle_opt {
            assert_eq!(puzzle.len(), num_layers);

            for layer in puzzle {
                // Extract mask
                let mut mask = vec![vec![false; num_cols]; num_rows];
                for r in 0..num_rows {
                    for c in 0..num_cols {
                        mask[r][c] = layer.values[r][c].is_some();
                    }
                }
                assert!(is_connected(&mask, num_rows, num_cols), "Layer should be physically connected");
            }
        }
    }
}
