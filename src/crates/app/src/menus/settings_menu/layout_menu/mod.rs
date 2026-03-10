use crate::tabs::TabName;
use egui_dock::DockState;
use serde::{Deserialize, Serialize};

pub mod layout;
pub use layout::*;
mod name_generator;

/// When a Dock isn't active, it's stored in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    pub dock_state: DockState<TabName>,
    // Whether this struct can be deleted. Exists to prevent accidental deletions from misclicks in the UI.
    pub locked: bool,
}
impl SavedLayout {
    #[must_use]
    pub fn from(dock_state: DockState<TabName>) -> Self {
        Self {
            dock_state,
            locked: false,
        }
    }
}

/// From the settings menu, this struct is copied to the clipboard and imported as a RON string.
/// This allows sharing UI layouts and easily swapping between presets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedLayout {
    pub name: String,
    pub dock: DockState<TabName>,
}
