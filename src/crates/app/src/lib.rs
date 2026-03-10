pub mod app;
pub mod log_categories;
pub mod menus;
pub mod tabs;
pub mod misc;
mod tick;

pub use app::MyApp;
pub use app::MyAppState;
pub use app::Layer;

/// The save directory name on native. "Local Storage" is used on the web.
/// The full directory depends on the system (Linux/etc). This const is expected to be used in `storage_dir(SAVE_DIR)`.
/// On Linux, it is ~/.local/share/my_app
///
/// All data from `impl eframe::App` gets put into this directory.
pub const SAVE_DIR: &str = "my_app"; // TEMPLATE_TODO: Change the name.
/// The file saved in the `SAVE_DIR`.
// Note that all storage goes into this key-value mapped file, similar to json.
// It includes keys not set by this app directly, like egui's internal state.
pub const SAVE_FILE: &str = "savedata.ron";

/// The number of ticks after which the save file will be copied to a backup file (on Desktop only).
/// This ensures that savedata corruption will not fully wipe the player's progress.
/// WARNING: This assumes there will be 100 ticks per second, which would result in 360,000 ticks per hour.
///   If this ends up being inaccurate, the autosave frequency will be very different.
pub const BACKUP_SAVE_TICK_INTERVAL: u128 = 360_000;
