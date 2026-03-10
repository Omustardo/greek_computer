use crate::MyAppState;
use crate::tabs::command::UiCommand;
use egui::Ui;
use egui_dock::OverlayType;

impl MyAppState {
    #[must_use]
    pub(crate) fn settings_menu_content(&mut self, ctx: &egui::Context, ui: &mut Ui) -> Vec<UiCommand> {
        self.show_graphics_settings(ctx, ui);
        let commands = self.show_layout_management_menu_button(ui);
        self.show_dock_settings(ui);
        self.show_controls_menu(ui);

        commands
    }
    fn show_graphics_settings(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        _ = ctx; // suppress unused variable that's only used in Native builds.

        ui.menu_button("Graphics", |ui| {
            // Fullscreen toggling doesn't work with this method on web. Users can use the
            // browser option (usually F11).
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Toggle Fullscreen").clicked() {
                    self.ui.is_fullscreen = !self.ui.is_fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.ui.is_fullscreen));
                }
            }

            egui::global_theme_preference_buttons(ui);
        });
    }

    fn show_dock_settings(&mut self, ui: &mut Ui) {
        let response = ui
            .menu_button("Dock", |ui| {
                ui.checkbox(&mut self.ui.dock_settings.spaced_tabs, "Spaced tabs").on_hover_text("Tab titles take up the entire available width");
                ui.checkbox(
                    &mut self.ui.dock_settings.show_close_buttons,
                    "Show close buttons on tabs",
                ).on_hover_text("Only when applicable. Not all tabs can be closed.");
                ui.checkbox(
                    &mut self.ui.dock_settings.show_leaf_close_all,
                    "Always show close buttons on tab bars",
                ).on_hover_text("All tabs within the tab bar must support closing to be able to close the full tab. Not all tabs can be closed, but most can.");
                ui.checkbox(
                    &mut self.ui.dock_settings.show_add_buttons,
                    "Show add buttons",
                );
                ui.checkbox(
                    &mut self.ui.dock_settings.show_leaf_collapse,
                    "Show collapse button on tab bars",
                );
                ui.checkbox(
                    &mut self.ui.dock_settings.show_top_bar,
                    "Show tab titlebar",
                );

                ui.horizontal(|ui| {
                    ui.label("Overlay Style:");
                    ui.radio_value(
                        &mut self.ui.dock_settings.overlay_type,
                        OverlayType::HighlightedAreas,
                        "Highlighted Areas",
                    );
                    ui.radio_value(
                        &mut self.ui.dock_settings.overlay_type,
                        OverlayType::Widgets,
                        "Widgets",
                    );
                })
                    .response
                    .on_hover_text("When dragging windows around, this changes the preview");
            })
            .response;
        response.on_hover_text("These options affect the behavior and appearance of tabs and the docking area.");
    }
}
