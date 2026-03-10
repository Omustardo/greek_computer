use crate::MyApp;
#[cfg(not(target_arch = "wasm32"))]
use crate::log_categories::LogCategory::Debug;
use chrono::{DateTime, Local};

impl MyApp {
    /// Render the File menu with cross-platform save operations
    pub(crate) fn render_file_menu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Open Binary Directory").clicked() {
                    self.open_directory(".");
                    ui.close();
                }
                if ui.button("Open Save Directory").clicked() {
                    self.open_save_directory();
                    ui.close();
                }
                ui.separator();
            }

            let save_info = if cfg!(target_arch = "wasm32") {
                "Save to browser storage"
            } else {
                "Save to file"
            };
            if ui
                .button(format!("Save (latest: {})", format_time_ago(self.state.session.save.latest_save_time)))
                .on_hover_text(save_info)
                .clicked()
            {
                // egui doesn't have a way to trigger a save on-demand, so hack it following https://github.com/emilk/egui/issues/5243
                // Set the autosave interval to zero, which will ensure that a save happens on the next game tick.
                self.state.session.save.needs_save = true;

                // WARNING: I don't expect it to happen, but if the save file gets bloated, this will cause notably UI lag.
                // Explicitly don't close the UI, so that the latest save time is easily confirmed)
                //ui.close();
            }
            if let Some(err) = &self.state.session.save.latest_save_error {
                ui.label(format!("WARNING: Latest save attempt failed: {err}"));
            }

            if ui
                .button("Export Save to Clipboard")
                .on_hover_text("Copy save data to clipboard for backup or sharing")
                .clicked()
            {
                self.export_save_to_clipboard(ctx);
                ui.close();
            }

            #[cfg(target_arch = "wasm32")]
            {
                if ui
                    .button("Save to File")
                    .on_hover_text("Download save file to your default download location.\nTo load a savefile, copy the file content and import it from clipboard")
                    .clicked()
                {
                    self.export_save_to_file();
                    ui.close();
                }
            }

            if ui
                .button("Import Save from Clipboard")
                .on_hover_text("Load save data from clipboard (⚠ overwrites current progress)")
                .clicked()
            {
                self.state.session.show_import_dialog = true;
                self.state.session.focus_import_dialog = true;
                ui.close();
            }

            #[cfg(debug_assertions)]
            {
                ui.separator();
                if ui
                    .button("Clear All Data")
                    .on_hover_text("Reset everything (including UI settings)")
                    .clicked()
                {
                    self.state.session.show_clear_confirmation = true;
                    ui.close();
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            }
        });

        self.render_import_dialog(ctx);

        #[cfg(debug_assertions)]
        self.render_clear_confirmation_dialog(ctx);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_directory(&mut self, path: &str) {
        if let Err(error) = open::that(path) {
            let message = if path == "." {
                format!("Failed to open current directory: {error}")
            } else {
                format!("Failed to open directory '{path}': {error}")
            };
            self.state.logger.log_debug(Debug, message);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_save_directory(&mut self) {
        let save_dir = self.get_save_directory();
        if let Some(save_dir) = save_dir {
            self.ensure_directory_exists(save_dir.as_str());
            self.open_directory(save_dir.as_str());
        } else {
            self.state.logger.log_error(
                Debug,
                format!(
                    "Failed to find save directory using eframe::storage_dir({:})",
                    crate::SAVE_DIR
                ),
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn ensure_directory_exists(&mut self, path: &str) {
        if let Err(e) = std::fs::create_dir_all(path) {
            self.state
                .logger
                .log_error(Debug, format!("Failed to create directory '{path}': {e}"));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_save_directory(&self) -> Option<String> {
        if let Some(dir) = eframe::storage_dir(crate::SAVE_DIR) {
            dir.to_str().map(|dir| dir.to_owned())
        } else {
            None
        }
    }
}

fn format_time_ago(then: DateTime<Local>) -> String {
    let duration = Local::now() - then;
    let seconds = duration.num_seconds();

    if seconds < 60 {
        format!("{seconds}s ago")
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs > 0 {
            format!("{mins}m {secs}s ago")
        } else {
            format!("{mins}m ago")
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{hours}h {mins}m ago")
        } else {
            format!("{hours}h ago")
        }
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours > 0 {
            format!("{days}d {hours}h ago")
        } else {
            format!("{days}d ago")
        }
    }
}
