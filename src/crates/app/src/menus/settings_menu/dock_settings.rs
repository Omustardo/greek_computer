use egui_dock::OverlayType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockSettings {
    pub show_top_bar: bool,
    pub show_close_buttons: bool,
    pub show_add_buttons: bool,
    pub show_leaf_close_all: bool,
    pub show_leaf_collapse: bool,
    pub overlay_type: OverlayType,
    pub spaced_tabs: bool,
}
