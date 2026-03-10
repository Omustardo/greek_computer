use crate::MyAppState;
use crate::log_categories::LogCategory;
use crate::log_categories::LogCategory::Debug;
use crate::menus::settings_menu::layout_menu::{ExportedLayout, SavedLayout};
use crate::tabs::{LayoutPresetName, TabName};
use egui_dock::{DockState, NodeIndex, SurfaceIndex};

/// Commands related to Dock / Layout / Tabs. Unlike most things in egui where actions can be
/// executed alongside normal logic, these commands need to be handled as a separate phase after
/// rendering is complete (or before it begins). The issue is with how egui_dock renders a DockArea.
///
/// See:
/// * https://gist.github.com/Omustardo/556fe3d0288740f46919b5b9b2533e69?permalink_comment_id=5723819#gistcomment-5723819
/// * https://gist.github.com/Omustardo/6bf42b80123dd07cd0375cc3fdd26286
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum UiCommand {
    /// Save the current layout using the given name. If there is an existing saved layout with the
    /// same name, the new layout will overwrite it. The "Default" layout cannot be modified.
    SaveLayout { name: String },
    /// Load one of the hard-coded preset layouts.
    LoadPreset { name: LayoutPresetName },
    /// Load a saved layout that has the given name. The current layout will be overwritten,
    /// so be sure to save it first if desired.
    LoadLayout { name: String },
    /// Delete a saved layout with the given name. If the layout does not exist, this does nothing.
    /// If the layout is locked, this does nothing.
    DeleteLayout { name: String },
    /// Export a layout with the given name to the user's clipboard.
    ExportLayout { name: String },
    /// Import a layout from the provided buffer.
    ImportLayout { data: String },
    /// Add a new tab to the specified location in the dock layout.
    /// If the location doesn't exist when this command is processed, this does nothing.
    AddTab {
        tab: TabName,
        surface: SurfaceIndex,
        node: NodeIndex,
    },
    /// Add a new tab to the parent tab's location. If the parent tab is not open, this does nothing.
    /// If multiple instances of the parent tab are open, the first one is used.
    AddTabOnParent { new_tab: TabName, parent_tab: TabName },
    /// Focus attempts to set focus to the provided tab. If the `tab_to_focus` isn't open, then nothing is done.
    Focus { tab_to_focus: TabName },
    /// FocusOrAdd attempts to set focus to the provided tab. If the `tab_to_focus` isn't open and
    /// parent_tab us, then `tab_to_focus` will be added to its location.
    /// If the parent_tab also isn't open, then nothing is done.
    FocusOrAdd { tab_to_focus: TabName, parent_tab: TabName },
    /// Set the protection status of a saved layout to prevent or allow deletion.
    /// If the layout does not exist, this does nothing.
    SetLayoutLock { name: String, locked: bool },
}

impl MyAppState {
    // NOTE: This could have been a function attached to MyApp since it needs to access so many fields.
    //   I weighed the option and ended up attaching it to MyAppState, with a reference to MyApp's
    //   current_dock.
    /// Apply all commands to the `current_dock_state`. This will do things like open tabs,
    /// focus on tabs, etc. Note that some handling of DockState is done internally within
    /// `egui_dock::DockArea::new(&mut dock_state).show_inside()`, such as people closing tabs
    /// with the "x" button in the corner of them.
    /// This is meant to handle dock modifications that are not standard `egui_dock`, such as
    /// saving and loading entire DockState from the settings menu.
    pub fn process_commands(
        &mut self,
        ctx: &egui::Context,
        current_dock_state: &mut DockState<TabName>,
        commands: &Vec<UiCommand>,
    ) {
        for command in commands {
            self.process_command(ctx, current_dock_state, command);
        }
    }

    fn process_command(
        &mut self,
        ctx: &egui::Context,
        current_dock_state: &mut DockState<TabName>,
        command: &UiCommand,
    ) {
        let session_state = &mut self.session.dock;
        let saved_layouts = &mut self.ui.saved_layouts;
        let current_layout_name = &mut self.ui.current_layout_name;

        // Limit the number of layouts for the sake of a usable UI.
        const MAX_LAYOUTS: usize = 16;
        match command {
            UiCommand::SaveLayout { name } => {
                if name == "Default" {
                    session_state.save_error_message = Some("Cannot modify the Default layout".to_string());
                    session_state.save_error_timer = Some(chrono::Local::now());
                    return;
                }

                if !saved_layouts.contains_key(name) && saved_layouts.len() >= MAX_LAYOUTS {
                    session_state.save_error_message =
                        Some(format!("No room for additional layouts. {MAX_LAYOUTS} is the maximum."));
                    session_state.save_error_timer = Some(chrono::Local::now());
                    return;
                }

                let saved_layout = SavedLayout {
                    dock_state: current_dock_state.clone(),
                    locked: false,
                };
                saved_layouts.insert(name.clone(), saved_layout);
                current_layout_name.clone_from(name);
                session_state.name_input_buf.clear();
                session_state.save_error_message = None;
                session_state.save_error_timer = None;
            }
            UiCommand::LoadLayout { name } => {
                if let Some(saved_layout) = saved_layouts.get(name) {
                    *current_dock_state = saved_layout.dock_state.clone();
                    *current_layout_name = name.clone();
                } else {
                    self.logger.log_error(
                        [Debug, LogCategory::Command],
                        format!("Didn't find layout named {name:}"),
                    );
                }
            }
            UiCommand::LoadPreset { name } => {
                *current_dock_state = name.dock_state();
                *current_layout_name = format!("{name:}");
            }
            UiCommand::DeleteLayout { name } => {
                if let Some(saved_layout) = saved_layouts.get(name) {
                    if !saved_layout.locked {
                        saved_layouts.remove(name);
                    }
                }
            }
            UiCommand::ExportLayout { name } => {
                if let Some(saved_layout) = saved_layouts.get(name) {
                    let exported = ExportedLayout {
                        name: name.clone(),
                        dock: saved_layout.dock_state.clone(),
                    };

                    match ron::ser::to_string_pretty(&exported, ron::ser::PrettyConfig::default()) {
                        Ok(ron_data) => {
                            ctx.copy_text(ron_data);
                            session_state.import_error_message = None;
                            session_state.import_error_timer = None;
                            session_state.export_success_message = Some("Layout copied to clipboard!".to_string());
                            session_state.export_success_timer = Some(chrono::Local::now());
                        }
                        Err(e) => {
                            session_state.import_error_message = Some(format!("Failed to export layout: {e}"));
                            session_state.import_error_timer = Some(chrono::Local::now());
                            session_state.export_success_message = None;
                            session_state.export_success_timer = None;
                        }
                    }
                }
            }
            UiCommand::ImportLayout { data } => {
                if data.trim().is_empty() {
                    session_state.import_error_message = Some("No layout data provided".to_string());
                    session_state.import_error_timer = Some(chrono::Local::now());
                    return;
                }

                if saved_layouts.len() >= MAX_LAYOUTS {
                    session_state.save_error_message =
                        Some(format!("No room for additional layouts. {MAX_LAYOUTS} is the maximum."));
                    return;
                }

                match ron::from_str::<ExportedLayout>(data) {
                    Ok(imported) => {
                        // If the import has the same name as an existing layout, rename it.
                        // Note that " #" numeric suffix also ensures that the names sorts
                        // alphabtically, so the new import shows up belo existing ones.
                        let original_name = imported.name.clone();
                        let mut final_name = imported.name;
                        let mut counter = 1;
                        while saved_layouts.contains_key(&final_name) {
                            final_name = format!("{original_name} #{counter:02X}");
                            counter += 1;
                        }

                        let saved_layout = SavedLayout {
                            dock_state: imported.dock.clone(),
                            locked: false,
                        };

                        saved_layouts.insert(final_name.clone(), saved_layout);
                        *current_dock_state = imported.dock;
                        *current_layout_name = final_name;

                        session_state.import_error_message = None;
                        session_state.import_error_timer = None;
                        session_state.export_success_message = None;
                        session_state.export_success_timer = None;
                    }
                    Err(e) => {
                        session_state.import_error_message = Some(format!("Failed to parse layout: {e}"));
                        session_state.import_error_timer = Some(chrono::Local::now());
                        session_state.export_success_message = None;
                        session_state.export_success_timer = None;
                    }
                }
            }
            UiCommand::AddTab { tab, surface, node } => {
                if current_dock_state.find_tab(tab).is_some() {
                    eprintln!("Attempted to add tab {tab:?} which is already active");
                    return;
                }
                current_dock_state[*surface].set_focused_node(*node);
                current_dock_state[*surface].push_to_focused_leaf(tab.clone());
            }
            UiCommand::AddTabOnParent { new_tab, parent_tab } => {
                if current_dock_state.find_tab(new_tab).is_some() {
                    eprintln!("Attempted to add tab {new_tab:?} which is already active");
                    return;
                }

                if let Some((surface, node, _)) = current_dock_state.find_tab(parent_tab) {
                    current_dock_state[surface].set_focused_node(node);
                    current_dock_state[surface].push_to_focused_leaf(new_tab.clone());
                } else {
                    eprintln!("Attempted to add tab {new_tab:?} to parent {parent_tab:?}, but parent tab wasn't found");
                }
            }
            UiCommand::Focus { tab_to_focus: tab } => {
                if let Some(found_tab) = current_dock_state.find_tab(tab) {
                    current_dock_state.set_active_tab(found_tab);
                } else {
                    eprintln!("Attempted to focus on tab {tab:?} but it wasn't found");
                }
            }
            UiCommand::FocusOrAdd {
                tab_to_focus: tab,
                parent_tab,
            } => {
                if let Some(found_tab) = current_dock_state.find_tab(tab) {
                    current_dock_state.set_active_tab(found_tab);
                } else {
                    // tab is not open, so open and focus on it
                    if let Some((surface, node, _)) = current_dock_state.find_tab(parent_tab) {
                        current_dock_state[surface].set_focused_node(node);
                        current_dock_state[surface].push_to_focused_leaf(tab.clone());
                    } else {
                        eprintln!("Attempted to add tab {tab:?} to parent {parent_tab:?}, but parent tab wasn't found");
                    }
                }
            }
            UiCommand::SetLayoutLock { name, locked } => {
                if let Some(layout) = saved_layouts.get_mut(name) {
                    layout.locked = *locked;
                }
            }
        }
    }
}
