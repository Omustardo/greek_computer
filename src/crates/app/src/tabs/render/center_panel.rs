// panel.rs

use crate::{MyAppState};
use egui::{Color32, Grid, RichText, ScrollArea, Ui, Vec2};

impl MyAppState {
    pub(crate) fn show_center_panel(&mut self, ui: &mut Ui) {
        ui.label("Rotate layers to make every column sum to 42.");
        ui.add_space(10.0);

        // 1. Calculate the combined state and column sums
        let (combined_grid, column_sums) = self.calculate_totals();

        // 2. Display the Combined Result (The "Stack")
        ui.group(|ui| {
            ui.heading("Combined View (Sum of all Layers)");
            ui.add_space(5.0);
            Self::draw_grid(ui, &combined_grid, Some(&column_sums));
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // 3. Display Individual Layers with Controls
        let mut pending_action = None;

        ScrollArea::vertical().show(ui, |ui| {
            // Using `enumerate()` is now possible because we defer mutation to the end
            for (i, layer) in self.state.layers.iter().enumerate() {
                ui.push_id(i, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Layer {}", i + 1)).strong());
                            ui.add_space(10.0);

                            // The default left-to-right flow prevents the group box
                            // from needlessly expanding to fill the parent width.
                            if ui.button("⏴ Rotate Left").clicked() {
                                pending_action = Some((i, -1));
                            }
                            if ui.button("Rotate Right ⏵").clicked() {
                                pending_action = Some((i, 1));
                            }
                        });

                        ui.add_space(5.0);
                        // Draw the individual layer
                        Self::draw_grid(ui, &layer.values, None);
                    });
                });
                ui.add_space(5.0);
            }
        });

        // Apply rotation after rendering if a button was clicked
        if let Some((layer_idx, dir)) = pending_action {
            self.rotate_layer(layer_idx, dir);
        }
    }

    /// Helper to calculate the flat grid sum and the column totals
    fn calculate_totals(&self) -> (Vec<Vec<Option<i16>>>, Vec<i32>) {
        let num_rows = 4;
        let num_cols = 12;

        let mut combined = vec![vec![None; num_cols]; num_rows];
        let mut col_sums = vec![0; num_cols];

        for layer in &self.state.layers {
            // Using `zip` guarantees we automatically limit bounds iteration without manual checks
            for (combined_row, layer_row) in combined.iter_mut().zip(&layer.values) {
                for ((combined_val, col_sum), &layer_val) in combined_row
                    .iter_mut()
                    .zip(col_sums.iter_mut())
                    .zip(layer_row)
                {
                    // If we encounter a number on this layer, check if we've already
                    // seen a number in a higher layer (is_none).
                    if let Some(v) = layer_val {
                        if combined_val.is_none() {
                            *combined_val = Some(v); // Store only the topmost visible number
                            *col_sum += i32::from(v);
                        }
                    }
                }
            }
        }

        (combined, col_sums)
    }

    /// Rotates a specific layer.
    /// Direction: -1 for Left, 1 for Right.
    fn rotate_layer(&mut self, layer_idx: usize, direction: i8) {
        if let Some(layer) = self.state.layers.get_mut(layer_idx) {
            for row in &mut layer.values {
                if direction > 0 {
                    row.rotate_right(1);
                } else if direction < 0 {
                    row.rotate_left(1);
                }
            }
        }
    }

    /// Renders a 4x12 grid of numbers.
    /// If `col_sums` is provided, it adds a footer row validating the totals against 42.
    fn draw_grid(ui: &mut Ui, grid_data: &[Vec<Option<i16>>], col_sums: Option<&[i32]>) {
        Grid::new("puzzle_grid")
            .striped(true)
            .min_col_width(25.0)
            .spacing(Vec2::new(10.0, 10.0))
            .show(ui, |ui| {
                // Draw Rows
                for row in grid_data {
                    for &val in row {
                        if let Some(n) = val {
                            ui.label(
                                RichText::new(format!("{:02}", n))
                                    .family(egui::FontFamily::Monospace)
                                    .color(ui.visuals().text_color()),
                            );
                        } else {
                            ui.label(RichText::new("__").color(Color32::from_gray(60)));
                        }
                    }
                    ui.end_row();
                }

                // Optional: Draw Column Sums (The Solution Check)
                if let Some(sums) = col_sums {
                    for &sum in sums {
                        let is_target = sum == 42;
                        let color = if is_target { Color32::GREEN } else { Color32::RED };

                        ui.label(
                            RichText::new(sum.to_string())
                                .strong()
                                .size(16.0)
                                .color(color),
                        );
                    }
                    ui.end_row();
                }
            });
    }
}