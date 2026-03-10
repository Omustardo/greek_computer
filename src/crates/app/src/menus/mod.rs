use crate::tabs::command::UiCommand;
use crate::{MyApp, MyAppState};
use egui::PopupCloseBehavior;
use egui::containers::menu::MenuConfig;

mod about_menu;
mod debug_menu;
mod main_menu;
pub mod settings_menu;

// MyApp needs to implement the menu (rather than MyAppState doing it). The File menu is used to
// save and load, which needs to touch the entire app and not just just MyAppState.
impl MyApp {
    pub(crate) fn menu_bar(&mut self, ctx: &egui::Context) -> Vec<UiCommand> {
        let mut commands = Vec::new();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.style_mut().compact_menu_style = true;

            egui::MenuBar::new()
                .config(self.state.create_menu_config())
                .ui(ui, |ui| {
                    self.render_file_menu(ctx, ui);
                    commands.extend(self.state.render_settings_menu(ctx, ui));
                    self.state.render_debug_menu_if_enabled(ctx, ui);
                    self.state.render_about_menu(ui);
                });
        });
        commands
    }
}

impl MyAppState {
    fn create_menu_config(&self) -> MenuConfig {
        // This is critical for complex menus. Without this, nearly any interaction
        // with a menu will close it, including opening a collapsed subsection.
        // See https://github.com/emilk/egui/pull/4636
        // For any menus that should close on click, it's easy to do this manually with ui.close()
        MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside)
    }

    fn render_settings_menu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Vec<UiCommand> {
        let mut commands = Vec::new();
        ui.menu_button("Settings", |ui| {
            commands.extend(self.settings_menu_content(ctx, ui));
        });
        commands
    }

    fn render_debug_menu_if_enabled(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // debug_assertions ensures that this menu doesn't exist if built with the --release flag.
        if cfg!(debug_assertions) {
            ui.menu_button("Debug", |ui| {
                self.debug_menu_content(ctx, ui);
            });
        }
    }
}
