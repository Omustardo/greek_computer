import sys

with open("src/crates/app/src/tabs/render/center_panel.rs", "r") as f:
    content = f.read()

ui_addition = """
        ui.group(|ui| {
            ui.heading("Generate New Puzzle");
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Layers:");
                ui.add(egui::DragValue::new(&mut self.session.gen_num_layers).range(1..=10));
                ui.add_space(10.0);

                ui.label("Rows:");
                ui.add(egui::DragValue::new(&mut self.session.gen_num_rows).range(1..=10));
                ui.add_space(10.0);

                ui.label("Columns:");
                ui.add(egui::DragValue::new(&mut self.session.gen_num_cols).range(1..=20));
                ui.add_space(10.0);

                ui.label("Target Sum:");
                ui.add(egui::DragValue::new(&mut self.session.gen_target_sum).range(1..=1000));
            });
            ui.add_space(5.0);
            if ui.button("Generate!").clicked() {
                self.state.layers = crate::puzzle_generator::generate_puzzle(
                    self.session.gen_num_layers,
                    self.session.gen_num_rows,
                    self.session.gen_num_cols,
                    self.session.gen_target_sum,
                );
                self.state.target_sum = self.session.gen_target_sum;
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
"""

# Find where to insert
insert_marker = 'ui.label(format!("Rotate layers to make every column sum to {}.", self.state.target_sum));\n        ui.add_space(10.0);\n'

content = content.replace(insert_marker, insert_marker + "\n" + ui_addition)

with open("src/crates/app/src/tabs/render/center_panel.rs", "w") as f:
    f.write(content)
