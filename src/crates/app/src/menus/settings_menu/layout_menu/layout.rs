use crate::MyAppState;
use crate::menus::settings_menu::layout_menu::SavedLayout;
use crate::menus::settings_menu::layout_menu::name_generator::TripletNameGenerator;
use crate::tabs::LayoutPresetName;
use crate::tabs::command::UiCommand;
use egui::{Ui, Widget};
use strum::IntoEnumIterator;

/// Ephemeral data used by the layout management menu.
#[derive(Default, Clone)]
pub struct DockSettingsSessionState {
    /// The name to use when saving the current layout.
    pub name_input_buf: String,
    pub save_error_message: Option<String>,
    pub save_error_timer: Option<chrono::DateTime<chrono::Local>>,

    /// If the user is doing it right, this will be a serialized SavedLayout.
    /// If it is in this field and they press the Load button, it will be added to
    /// the list of saved layouts.
    pub data_import_buf: String,
    pub import_error_message: Option<String>,
    pub import_error_timer: Option<chrono::DateTime<chrono::Local>>,

    pub export_success_message: Option<String>,
    pub export_success_timer: Option<chrono::DateTime<chrono::Local>>,
}

impl MyAppState {
    #[must_use]
    pub(crate) fn show_layout_management_menu_button(&mut self, ui: &mut Ui) -> Vec<UiCommand> {
        let mut commands: Vec<UiCommand> = Vec::new();
        ui.menu_button("Layouts", |ui| {
            self.show_current_layout_info(ui);
            commands.extend(self.show_save_section(ui));
            commands.extend(self.show_reset_section(ui));
            commands.extend(self.show_import_section(ui));
            commands.extend(self.show_saved_layouts_section(ui));

            ui.separator();
            if let Some(error_msg) = &self.session.dock.import_error_message {
                ui.colored_label(egui::Color32::RED, error_msg);
            }
            if let Some(error_msg) = &self.session.dock.save_error_message {
                ui.colored_label(egui::Color32::RED, error_msg);
            }
            if let Some(success_msg) = &self.session.dock.export_success_message {
                ui.colored_label(egui::Color32::GREEN, success_msg);
            }
        });
        self.update_message_timers();
        commands
    }

    fn show_current_layout_info(&mut self, ui: &mut Ui) {
        // Use this label constructor rather than `ui.Label()` to ensure
        // the UI doesn't get into a bad state based on bad user input.
        // For example, pasting the exported layout into the name field
        // and "accidentally" saving it would make the UI so tall that
        // it wouldn't be possible to delete the offending layout!
        //
        // This doesn't sanitize the underlying name, so if the user
        // wants to store a massive unreadable string in this field, it is left
        // up to their disgression.
        egui::widgets::Label::new(format!("Current Layout: {}", self.ui.current_layout_name))
            .truncate()
            .ui(ui);
        ui.separator();
    }

    #[must_use]
    fn show_save_section(&mut self, ui: &mut Ui) -> Vec<UiCommand> {
        let mut commands = Vec::new();
        if ui.button("Save Current Layout").clicked() {
            commands.push(UiCommand::SaveLayout {
                name: self.ui.current_layout_name.clone(),
            });
            // Explicitly don't exit the menu. The user may want to export or do some other action.
            // ui.close();
        }
        // Input area for the name of the current layout. Used when clicking the button above.
        ui.horizontal(|ui| {
            if ui.button("Save Current Layout as:").clicked() {
                let name = if self.session.dock.name_input_buf.trim().is_empty() {
                    // TODO: Ensure that this doesn't overwrite any existing layouts. It's unlikely, but not impossible for it to happen by chance.
                    TripletNameGenerator::generate_unique(&self.ui.saved_layouts)
                } else {
                    self.session.dock.name_input_buf.trim().to_string()
                };
                commands.push(UiCommand::SaveLayout { name });
            }

            ui.add(
                egui::TextEdit::singleline(&mut self.session.dock.name_input_buf)
                    .hint_text("Layout name, or empty for generated"),
            )
        });
        commands
    }

    #[must_use]
    fn show_import_section(&mut self, ui: &mut Ui) -> Vec<UiCommand> {
        let mut commands = Vec::new();

        ui.horizontal(|ui| {
            if ui.button("Import Layout data:").clicked() {
                commands.push(UiCommand::ImportLayout {
                    data: self.session.dock.data_import_buf.clone(),
                });
            }
            ui.add(
                egui::TextEdit::singleline(&mut self.session.dock.data_import_buf).hint_text("Paste layout data here"),
            )
        });
        ui.separator();
        commands
    }

    #[must_use]
    fn show_reset_section(&mut self, ui: &mut Ui) -> Vec<UiCommand> {
        let mut commands = Vec::new();

        ui.horizontal(|ui| {
            ui.label("Load Preset: ");
            for name in LayoutPresetName::iter() {
                if ui.button(format!("{name}")).clicked() {
                    commands.push(UiCommand::LoadPreset { name });
                    // Explicitly don't close. It's annoying.
                    // ui.close();
                }
            }
        });

        ui.separator();
        commands
    }

    #[must_use]
    fn show_saved_layouts_section(&mut self, ui: &mut Ui) -> Vec<UiCommand> {
        if self.ui.saved_layouts.is_empty() {
            ui.label("No saved layouts");
            return Vec::new();
        }
        ui.label("Saved Layouts:");
        let mut commands = Vec::new();

        // Clone to avoid borrow checker issues
        let mut saved_layouts: Vec<(String, SavedLayout)> = self
            .ui
            .saved_layouts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // Sort by the layout name. This is known to be unique since it just came from a HashMap.
        saved_layouts.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, saved_layout) in saved_layouts {
            ui.horizontal(|ui| {
                ui.set_min_width(200.0);

                if ui.button("Load").clicked() {
                    commands.push(UiCommand::LoadLayout { name: name.clone() });
                    // I explicitly don't close the UI here because I don't
                    // like that interaction.
                    // ui.close();
                }

                egui::widgets::Label::new(name.clone()).truncate().ui(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let delete_enabled = !saved_layout.locked;
                    if ui
                        .add_enabled(delete_enabled, egui::Button::new("Delete"))
                        .on_disabled_hover_text("Unprotect to delete")
                        .clicked()
                    {
                        commands.push(UiCommand::DeleteLayout { name: name.clone() });
                    }

                    let mut locked = saved_layout.locked;
                    if ui
                        .checkbox(&mut locked, "Protect")
                        .on_hover_text("Protect from deletion")
                        .changed()
                    {
                        commands.push(UiCommand::SetLayoutLock {
                            name: name.clone(),
                            locked,
                        });
                    }

                    if ui.button("Export").on_hover_text("Copy to clipboard").clicked() {
                        commands.push(UiCommand::ExportLayout { name: name.clone() });
                    }
                });
            });
        }
        commands
    }

    fn update_message_timers(&mut self) {
        if let Some(timer) = self.session.dock.export_success_timer {
            if chrono::Local::now().signed_duration_since(timer) > chrono::TimeDelta::seconds(3) {
                self.session.dock.export_success_message = None;
                self.session.dock.export_success_timer = None;
            }
        }
        if let Some(timer) = self.session.dock.import_error_timer {
            if chrono::Local::now().signed_duration_since(timer) > chrono::TimeDelta::seconds(5) {
                self.session.dock.import_error_message = None;
                self.session.dock.import_error_timer = None;
            }
        }
        if let Some(timer) = self.session.dock.save_error_timer {
            if chrono::Local::now().signed_duration_since(timer) > chrono::TimeDelta::seconds(5) {
                self.session.dock.save_error_message = None;
                self.session.dock.save_error_timer = None;
            }
        }
    }
}
