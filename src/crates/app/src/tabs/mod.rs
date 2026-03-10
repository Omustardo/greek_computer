pub mod command;
mod layout_presets;
mod my_tab_viewer;
mod render;

pub use command::*;
use egui_dock::DockState;
pub use layout_presets::*;
pub use my_tab_viewer::MyAppTabViewer;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Ord, PartialOrd, Display)]
#[strum(serialize_all = "title_case")]
pub enum TabName {
    // Sort alphabetically so that the default iteration order is pleasing to the eye.
    // keep-sorted start
    CenterPanel,
    // keep-sorted end
}

impl TabName {
    #[must_use]
    pub fn display_name(&self) -> String {
        self.to_string()
    }

    pub(crate) fn is_closeable(&self) -> bool {
        // WARNING: There needs to be at least one tab that isn't closable, or all might be closed
        //   accidentally. This isn't a major issue since the menu allows resetting to a default
        //   layout. Keeping Logs always open seems fine.
        !matches!(self, TabName::CenterPanel)
    }
}

#[must_use]
pub fn get_open_tabs(dock_state: &DockState<TabName>) -> Vec<TabName> {
    dock_state
        .iter_all_tabs()
        .map(|(_, tab_name)| tab_name.clone())
        .collect()
}
#[must_use]
pub fn get_closed_tabs(dock_state: &DockState<TabName>) -> Vec<TabName> {
    let open_tabs: std::collections::HashSet<_> = get_open_tabs(dock_state).into_iter().collect();
    TabName::iter().filter(|tab| !open_tabs.contains(tab)).collect()
}
