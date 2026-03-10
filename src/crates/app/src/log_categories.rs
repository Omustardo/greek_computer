use std::fmt;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LogCategory {
    #[default]
    UNKNOWN,
    /// Anything that would be helpful for a developer to debug an issue.
    Debug,
    /// A command issued by the human playing the game. For example, exporting a savegame
    Command,
    // TODO: Add more categories as needed.
}
impl fmt::Display for LogCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl egui_logger::IntoCategories for LogCategory {
    fn into_categories(self) -> Vec<String> {
        vec![self.to_string()]
    }
}
