use crate::MyAppState;
use crate::tabs::TabName;
use crate::tabs::command::UiCommand;
use egui::{Id, Response, Ui, WidgetText};
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{NodeIndex, SurfaceIndex, TabViewer};

// A custom TabViewer that has access to all of the app state, as well as
// access to pre-computed information about DockState (the `available_tabs`).
// Since this struct cannot have a reference to DockState (it would cause borrow issues),
// it populates `commands`.
// Commands then need to be handled by the caller after `DockArea::new(dock_state)::show(tab_viewer);`
pub struct MyAppTabViewer<'a> {
    pub state: &'a mut MyAppState,
    pub available_tabs: Vec<TabName>,
    pub commands: &'a mut Vec<UiCommand>,
}

impl TabViewer for MyAppTabViewer<'_> {
    type Tab = TabName;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.display_name().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.ctx().all_styles_mut(|style| {
            // Remove on-hover delays. Delays are a bad user experience.
            // There may to be a way to set this in a startup option, but I didn't find the way.
            style.interaction.tooltip_delay = 0.0;
            // Remove a debug option to show when integer division makes pixels unaligned.
            // It displays "Show Unaligned" in affected widgets.
            // I couldn't see the unalignment, so I don't think it matters.
            // https://docs.rs/egui/latest/egui/style/struct.DebugOptions.html#structfield.show_unaligned
            #[cfg(debug_assertions)]
            {
                style.debug.show_unaligned = false;
            }
        });

        pub use TabName::*;
        ui.push_id(format!("tab_{tab:?}"), |ui| match tab {
            CenterPanel => self.state.show_center_panel(ui),
        });
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        // Explicitly choose the name as the ID. The default is to use the title, which causes
        // inconsistencies when using short_tab_names.
        Id::from(tab.display_name())
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &Response) {
        if response.hovered() {
            egui::containers::Tooltip::for_widget(response).show(|ui| {
                ui.label(tab.display_name());
            });
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> OnCloseResponse {
        OnCloseResponse::Close
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        tab.is_closeable()
    }

    fn on_add(&mut self, _surface: SurfaceIndex, _node: NodeIndex) {
        // This method is called when the "+" button is clicked, but we use add_popup instead to show a selection menu.
    }

    /// This content is rendered in a menu / popup when the user interacts with the "+" button in
    /// the top-right of a egui_dock node. It lists which tabs are available to open.
    fn add_popup(&mut self, ui: &mut Ui, surface: SurfaceIndex, node: NodeIndex) {
        if self.available_tabs.is_empty() {
            // This state should never be reached, but handle it to be safe.
            // Look for usage of `show_add_buttons` to see how the add button is disabled when
            // all tabs are open.
            ui.label("All tabs are open");
            return;
        }
        for tab in &self.available_tabs {
            if ui.button(tab.display_name()).clicked() {
                self.commands.push(UiCommand::AddTab {
                    tab: tab.clone(),
                    surface,
                    node,
                });
                ui.close();
            }
        }
    }
}
