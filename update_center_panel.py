import sys

with open("src/crates/app/src/tabs/render/center_panel.rs", "r") as f:
    content = f.read()

# Update show_center_panel to use target_sum
old_show_center_panel_str1 = 'ui.label("Rotate layers to make every column sum to 42.");'
new_show_center_panel_str1 = 'ui.label(format!("Rotate layers to make every column sum to {}.", self.state.target_sum));'
content = content.replace(old_show_center_panel_str1, new_show_center_panel_str1)

# Update calculate_totals
old_calc_totals = """    fn calculate_totals(&self) -> (Vec<Vec<Option<i16>>>, Vec<i32>) {
        let num_rows = 4;
        let num_cols = 12;"""

new_calc_totals = """    fn calculate_totals(&self) -> (Vec<Vec<Option<i16>>>, Vec<i32>) {
        let num_rows = self.state.layers.first().map_or(0, |l| l.values.len());
        let num_cols = self.state.layers.first().map_or(0, |l| l.values.first().map_or(0, |r| r.len()));"""
content = content.replace(old_calc_totals, new_calc_totals)

# Update draw_grid to use target_sum
old_draw_grid_str = """    fn draw_grid(ui: &mut Ui, grid_data: &[Vec<Option<i16>>], col_sums: Option<&[i32]>) {"""
new_draw_grid_str = """    fn draw_grid(ui: &mut Ui, grid_data: &[Vec<Option<i16>>], col_sums: Option<&[i32]>, target_sum: i32) {"""
content = content.replace(old_draw_grid_str, new_draw_grid_str)

old_target_check = """                    for &sum in sums {
                        let is_target = sum == 42;"""
new_target_check = """                    for &sum in sums {
                        let is_target = sum == target_sum;"""
content = content.replace(old_target_check, new_target_check)

# Update calls to draw_grid
old_call1 = "Self::draw_grid(ui, &combined_grid, Some(&column_sums));"
new_call1 = "Self::draw_grid(ui, &combined_grid, Some(&column_sums), self.state.target_sum);"
content = content.replace(old_call1, new_call1)

old_call2 = "Self::draw_grid(ui, &layer.values, None);"
new_call2 = "Self::draw_grid(ui, &layer.values, None, self.state.target_sum);"
content = content.replace(old_call2, new_call2)

with open("src/crates/app/src/tabs/render/center_panel.rs", "w") as f:
    f.write(content)
