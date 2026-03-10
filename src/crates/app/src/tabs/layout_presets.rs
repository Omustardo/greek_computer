use crate::tabs::TabName;
use egui_dock::{DockState};
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, EnumIter, Display)]
pub enum LayoutPresetName {
    SinglePanel,
}

impl LayoutPresetName {
    #[must_use]
    pub fn dock_state(&self) -> DockState<TabName> {
        match self {
            LayoutPresetName::SinglePanel => only_center(),
        }
    }
}

fn only_center() -> DockState<TabName> {
    DockState::new(vec![TabName::CenterPanel])
}


// fn initial_attempt() -> DockState<TabName> {
//     use TabName::{LeftPanel, CenterPanel};
//
//     // Start with Widgets in the main area
//     let mut state = DockState::new(vec![Location, Artifacts]);
//
//     // Split left: Widgets stays on right, Locations goes to new left area
//     let [_widgets_area, mut current_left] = state.main_surface_mut().split_left(NodeIndex::root(), 0.2, vec![Map]);
//
//     // Add remaining left sidebar tabs with equal spacing
//     let left_tabs = vec![Stats, Skills, Equipment, LeftPanel];
//     for (i, tab) in left_tabs.into_iter().enumerate() {
//         // For equal distribution, calculate the ratio of remaining space.
//         // It is extremely important to not ever have a ratio of 1.0
//         // as that causes a known bug in egui_dock: https://github.com/Adanos020/egui_dock/issues/282
//         let ratio = 1.0 / (4 - i + 1) as f32; // 1/5, 1/4, 1/3, 1/2
//         let [_prev_area, new_area] = state.main_surface_mut().split_below(current_left, ratio, vec![tab]);
//         current_left = new_area;
//     }
//
//     // Split the right side to add Logs below Widgets
//     let [_widgets_area, _logs_area] = state.main_surface_mut().split_below(NodeIndex::root(), 0.7, vec![CenterPanel]);
//
//     state
// }
