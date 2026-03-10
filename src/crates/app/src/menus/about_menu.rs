use crate::MyAppState;

const LICENSES: &str = include_str!("../../../../../LICENSES");

impl MyAppState {
    pub(crate) fn render_about_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("About", |ui| {
            ui.menu_button("Build", |ui| {
                self.render_powered_by_message(ui);
                self.render_licenses(ui);
                egui::warn_if_debug_build(ui);
            });

            // TEMPLATE_TODO: Set your Feedback page or delete this.
            // ui.menu_button("Feedback", |ui| {
            //     ui.hyperlink("https://www.omustardo.com#feedback");
            // });
        });
    }

    fn render_licenses(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Licenses", |ui| {
            egui::ScrollArea::new([true, true]).max_height(1000.0).show(ui, |ui| {
                ui.label(LICENSES);
            });
        });
    }

    fn render_powered_by_message(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Powered by ");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label(" and ");
            ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
            ui.label(".");
        });
    }
}
