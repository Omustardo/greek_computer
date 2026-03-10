use crate::MyApp;
use egui::vec2;

impl MyApp {
    /// Render the import save file dialog
    pub(crate) fn render_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.state.session.show_import_dialog {
            return;
        }

        egui::Window::new("Import Save File")
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 350.0])
            .min_size([400.0, 250.0])
            .max_size([600.0, 500.0])
            .show(ctx, |ui| {
                // Warning at the top
                ui.horizontal(|ui| {
                    ui.label("⚠");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 150, 0), // Orange warning color
                        "Warning: This will overwrite your current data and all settings!",
                    );
                });

                ui.separator();
                ui.label("Paste your save file content below:");

                // Use available space for the text area, leaving room for buttons and labels
                let available_rect = ui.available_rect_before_wrap();
                let button_height = 30.0;
                let spacing = 60.0; // More space for warning and labels
                let text_area_height = (available_rect.height() - button_height - spacing).max(120.0);

                egui::ScrollArea::vertical()
                    .max_height(text_area_height)
                    .show(ui, |ui| {
                        // Setting a size is critical. Otherwise the large amount of text input
                        // will make the text box larger than the visible region, which prevents
                        // it from being closed.
                        let resp = ui.add_sized(
                            vec2(available_rect.width() - 20.0, text_area_height),
                            egui::TextEdit::multiline(&mut self.state.session.import_text_buffer)
                                .desired_rows(10)
                                .font(egui::TextStyle::Monospace),
                        );
                        if self.state.session.focus_import_dialog {
                            // Focus the input on the new text box.
                            resp.request_focus();
                            // Don't grab focus again. Once is enough.
                            // Grabbing focus permanently would be bad.
                            self.state.session.focus_import_dialog = false;
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    // Import button with warning styling
                    let import_button =
                        egui::Button::new("⚠ Import & Overwrite").fill(egui::Color32::from_rgb(200, 100, 100)); // Reddish to indicate danger

                    if ui.add(import_button).clicked() {
                        self.import_save_from_text(ctx);
                        self.state.session.show_import_dialog = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.state.session.import_text_buffer.clear();
                        self.state.session.show_import_dialog = false;
                    }
                });

                ui.separator();
                ui.small("Tip: Export your current save first as a backup");
                ui.small("Paste the contents of your save file above");
            });
    }

    /// Render the clear save data confirmation dialog. It only exists in debug mode, since normal
    /// users shouldn't ever need to clear their data. If they do, they can delete the save file.
    #[cfg(debug_assertions)]
    pub(crate) fn render_clear_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if !self.state.session.show_clear_confirmation {
            return;
        }

        egui::Window::new("Clear Save Data")
            .collapsible(false)
            .resizable(false)
            .default_size([350.0, 200.0])
            .show(ctx, |ui| {
                // Warning header
                ui.horizontal(|ui| {
                    ui.label("⚠");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100), // Red warning color
                        "Permanent Action Warning",
                    );
                });

                ui.separator();

                ui.label("Are you sure you want to clear all in-memory data?");
                ui.colored_label(egui::Color32::from_rgb(255, 150, 0), "This action cannot be undone!");

                ui.separator();
                ui.label("This will reset everything.");
                ui.label("Consider exporting your save first");

                ui.separator();

                ui.horizontal(|ui| {
                    // Dangerous action button
                    let clear_button =
                        egui::Button::new("Yes, clear everything").fill(egui::Color32::from_rgb(180, 60, 60)); // Red to indicate danger

                    if ui.add(clear_button).clicked() {
                        self.clear_all_data();
                        self.state.session.show_clear_confirmation = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.state.session.show_clear_confirmation = false;
                    }
                });
            });
    }
}
