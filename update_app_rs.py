import sys

def replace_between(content, start_marker, end_marker, replacement):
    start = content.find(start_marker)
    if start == -1: return content
    end = content.find(end_marker, start)
    if end == -1: return content
    return content[:start] + replacement + content[end:]

with open("src/crates/app/src/app.rs", "r") as f:
    content = f.read()

# Update SessionState
session_state_insert = """
    pub save: SavingState,

    // Generation configuration
    pub gen_num_layers: usize,
    pub gen_num_rows: usize,
    pub gen_num_cols: usize,
    pub gen_target_sum: i32,
"""

content = content.replace("    pub save: SavingState,", session_state_insert)

# Update GameState
game_state_insert = """#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct GameState {
    pub layers: Vec<Layer>,
    pub target_sum: i32,
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            layers: Layer::default_layers(),
            target_sum: 42,
        }
    }
}"""

# Actually, I will replace the GameState block
start_str = "#[derive(serde::Deserialize, serde::Serialize, Clone)]\npub struct GameState {"
end_str = "impl Default for MyAppState {"

content = replace_between(content, start_str, end_str, game_state_insert + "\n\n")

# Update SessionState Default
session_state_default_insert = """            session: {
                let mut sess = SessionState::default();
                sess.save.latest_save_time = chrono::Local::now();
                sess.gen_num_layers = 5;
                sess.gen_num_rows = 4;
                sess.gen_num_cols = 12;
                sess.gen_target_sum = 42;
                sess
            },"""

content = content.replace("""            session: {
                let mut sess = SessionState::default();
                sess.save.latest_save_time = chrono::Local::now();
                sess
            },""", session_state_default_insert)

with open("src/crates/app/src/app.rs", "w") as f:
    f.write(content)
