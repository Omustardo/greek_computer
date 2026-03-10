
use crate::log_categories::LogCategory;
use crate::menus::settings_menu::dock_settings::DockSettings;
use crate::menus::settings_menu::layout_menu::DockSettingsSessionState;
use crate::menus::settings_menu::layout_menu::SavedLayout;
use crate::tabs::{LayoutPresetName, MyAppTabViewer, TabName, get_closed_tabs};
use chrono;
use chrono::{Duration, TimeDelta};
use egui::{CornerRadius, RawInput};
use egui_dock::{DockState, OverlayType};
use egui_logger::{EguiLogger, TimeFormat, TimePrecision};
use std::collections::HashMap;
use std::ops::Div;
use crate::misc::fps_counter::FpsCounter;

/// MyApp exists for clean program structure. Putting dock_state within MyAppState
/// doesn't work due to a borrow reference when using egui_dock. In order to render a UI
/// with multiple resizeable tabs in egui_dock, you must do:
/// `DockArea::new(&mut self.dock_state).show(&mut tab_viewer);`
/// tab_viewer is anything that implements the TabViewer trait, but is generally whatever holds
/// your app's state, since that is presumably what needs to be shown. If tab_viewer were `self`
/// or if it held DockState anywhere inside of it, it would be non-trivial to use it without borrow
/// issues.
//
// NOTE: If any fields or sub-fields are renamed, they must set `#[serde(alias = "old_field_name")]`
//   or handled in a custom
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MyApp {
    pub(crate) state: MyAppState,
    /// AVOID MODIFYING THIS DIRECTLY! See [`MyAppState::process_commands`].
    /// It is exposed for save/load serialization purposes only.
    pub(crate) dock_state: DockState<TabName>,
    // TODO: An identifier for when this struct was created. Needed if you want to support a new binary loading old/incompatible saves.
    // TODO: Add functions for migrating from older states to newer states. For example, moving values from old fields to new ones.
    // TODO: Look up the rules for how RON files are loaded when unexpected fields are found. Is that whole field cleared?
    version: u64,
}

/// The central struct that holds almost all state. egui has some UI state that it manages and we don't touch.
#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MyAppState {
    pub state: GameState,
    pub ui: UiState,
    pub tick: TickState,
    #[serde(skip)]
    pub session: SessionState,

    pub logger: EguiLogger,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct UiState {
    window_name: String,
    pub(crate) is_fullscreen: bool,

    pub current_layout_name: String,
    pub saved_layouts: HashMap<String, SavedLayout>,

    pub dock_settings: DockSettings,
}

/// A grab bag of variables that don't need to be saved between sessions. A session refers to running the game binary.
/// This is a lot of temporary UI state that is only used in a single UI.
#[derive(Default, Clone)]
pub struct SessionState {
    /// Whether to show the window for importing save data.
    pub show_import_dialog: bool,
    /// Whether to grab focus onto the save import textbox. This is expected to
    /// be set to true when show_import_dialog is first set to true. Once focus has been
    /// moved it must be set to false so that it doesn't permanently grab focus.
    pub focus_import_dialog: bool,
    pub import_text_buffer: String,

    // The option to clear all in-memory state is only needed for debug builds.
    #[cfg(debug_assertions)]
    pub show_clear_confirmation: bool,

    pub license_search_query: String,
    pub fps_counter: FpsCounter,
    pub dock: DockSettingsSessionState,
    /// When the save file was last attempted to be backed up to a `.bak` file.
    /// This should arguably be part of `MyApp`, but there isn't other
    /// sessionized data there, so this seems cleaner.
    /// Autosaves happen regularly (every 15s as of 2025-08), but it's important
    /// to make a backup of the save in order to prevent total loss if the
    /// savefile were somehow corrupted.
    pub latest_backup_tick: u128,

    pub save: SavingState,
}

/// Variables related to saving the game.
#[derive(Clone, Default)]
pub struct SavingState {
    /// Setting this to true triggers egui to save the game on the next tick.
    /// This value will then be set to false.
    pub needs_save: bool,
    /// When the game was most recently saved. This is initialized to the current time when
    /// a game is first loaded from file.
    pub latest_save_time: chrono::DateTime<chrono::Local>,
    /// The error message for the latest failed save attempt. This is cleared when a save succeeds.
    pub latest_save_error: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TickState {
    /// When the last save or autosave occurred, to ensure autosaves don't happen too quickly.
    /// TODO: Use this or remove it.
    latest_save_time: chrono::DateTime<chrono::Local>,
    /// The timestamp when `update` was most recently called.
    last_update_time: chrono::DateTime<chrono::Local>,
    /// The duration between calls to `update` that has not yet been used for game ticks.
    accumulated_delta_time: Duration,

    /// The total duration of time that this application has been active for.
    pub time_elapsed_total: Duration,

    /// The duration between game ticks. This is often easier to think about as ticks per second.
    /// To use ticks per second, set this value by doing "1 second / desired_ticks_per_second" like:
    /// TimeDelta::new(1, 0).unwrap().div(100)
    pub target_tick_interval: TimeDelta,
    /// ticks_active is whether the game should progress. UI actions are still possible. This is meant for debugging.
    pub ticks_active: bool,
    /// The total number of game ticks that have been processed, which includes bonus ticks.
    pub ticks_processed_total: u128,
    /// The number of game ticks that have been processed since the last reset, including bonus ticks.
    pub ticks_processed_current_loop: u128,
}
impl TickState {
    pub fn on_reset(&mut self) {
        self.ticks_processed_current_loop = 0;
    }
}

const NUM_ROWS: usize = 4;
const NUM_COLS: usize = 12;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Layer {
    pub values: Vec<Vec<Option<i16>>>
}
impl Layer {
    pub fn new(values: Vec<Vec<Option<i16>>>) -> Layer {
        Layer { values }
    }
    pub fn new_from_text(num_rows: usize, num_cols: usize, txt: &str) -> Layer {
        let mut values = vec![vec![None; num_cols]; num_rows];

        // Create an iterator that ignores whitespace/newlines
        let mut tokens = txt.split_whitespace();

        for row in 0..num_rows {
            for col in 0..num_cols {
                let token = tokens.next()
                    .expect("Input text does not contain enough tokens to fill the layer");

                values[row][col] = match token {
                    "__" => None,
                    val => Some(val.parse::<i16>().expect("Invalid number format encountered")),
                };
            }
        }

        Layer { values }
    }
    pub fn default_layers() -> Vec<Layer> {
        vec![
            Layer::new_from_text(NUM_ROWS, NUM_COLS, "
__ __ __ __ __ __ __ __ __ __ __ __
__ __ __ __ __ __ __ __ __ __ __ __
__ __ __ __ __ __ __ __ __ __ __ __
10 __ 07 __ 15 __ 08 __ 03 __ 06 __"),

            Layer::new_from_text(NUM_ROWS, NUM_COLS, "
__ __ __ __ __ __ __ __ __ __ __ __
__ __ __ __ __ __ __ __ __ __ __ __
__ 14 __ 09 __ 12 __ 04 __ 07 15 __
11 11 06 11 __ 06 17 07 03 __ 06 __"),

            Layer::new_from_text(NUM_ROWS, NUM_COLS, "
__ __ __ __ __ __ __ __ __ __ __ __
__ 09 __ 05 __ 10 __ 08 __ 22 __ 16
01 12 __ 21 06 15 04 09 18 11 26 14
__ 07 08 09 13 09 07 13 21 17 04 05"),

            Layer::new_from_text(NUM_ROWS, NUM_COLS, "
12 __ 06 __ 10 __ 10 __ 01 __ 09 __
02 13 09 __ 17 19 03 12 03 26 06 __
06 __ 14 12 03 08 09 __ 09 20 12 03
07 14 11 __ 08 __ 16 02 07 __ 09 __"),

            Layer::new_from_text(NUM_ROWS, NUM_COLS, "
08 03 04 12 02 05 10 07 16 08 07 08
04 04 06 06 03 03 14 14 21 21 09 09
04 05 06 07 08 09 10 11 12 13 14 15
11 11 14 11 14 11 14 14 11 14 11 14")
            ]
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct GameState {
    pub layers: Vec<Layer>
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            layers: Layer::default_layers()
        }
    }
}

impl Default for MyAppState {
    fn default() -> Self {
        // Initialize the logger
        let mut logger = EguiLogger::new();
        logger.show_categories = false;
        logger.show_level = false;
        logger.time_format = TimeFormat::LocalTime;
        logger.time_precision = TimePrecision::Seconds;

        // Hide the Settings and Input to begin. The fewer UI elements we start with, the better.
        // There is also no immediate need for being able to enter text.
        logger.show_settings = false;
        logger.show_input_area = false;

        logger.log_info(LogCategory::UNKNOWN, "-----------------------");
        Self {
            state: GameState::default(),
            ui: UiState {
                // TEMPLATE_TODO: Change the window_name.
                window_name: "My App".to_owned(),
                is_fullscreen: false,
                dock_settings: DockSettings {
                    show_top_bar: false,
                    show_leaf_close_all: false,
                    show_leaf_collapse: false,
                    show_close_buttons: false,
                    show_add_buttons: true,
                    overlay_type: OverlayType::Widgets,
                    spaced_tabs: true,
                },
                current_layout_name: "Default".to_string(),
                saved_layouts: HashMap::new(),
            },
            tick: TickState {
                latest_save_time: chrono::Local::now(),
                last_update_time: Default::default(),
                accumulated_delta_time: Default::default(),
                time_elapsed_total: Default::default(),
                target_tick_interval: TimeDelta::new(1, 0).unwrap().div(100), // Aim for 100 ticks per second.
                ticks_active: true,
                ticks_processed_total: 0,
                ticks_processed_current_loop: 0,
            },
            session: {
                let mut sess = SessionState::default();
                sess.save.latest_save_time = chrono::Local::now();
                sess
            },
            logger,
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: MyAppState::default(),
            dock_state: LayoutPresetName::SinglePanel.dock_state(),
            version: 0,
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // NOTE: Consider using a custom font. This may not be trivial because:
        //  1. The font might not support all characters, such as a right-facing arrow used
        //     in the egui_logger's filter menu. https://github.com/emilk/egui/issues/3529
        //  2. Widget sizes might need to be adjusted.
        Self::setup_fonts(cc);

        // Load previous app state (if any).

        if let Some(storage) = cc.storage {
            Self::load_from_storage(storage).unwrap_or_else(MyApp::default)
        } else {
            MyApp::default()
        }
    }

    /// If enough ticks have elapsed, attempt to copy the current save file to a backup file.
    ///
    /// WARNING: Backup saves are only supported on Desktop, since there just isn't an obvious place to put them in Web storage.
    /// This function is expected to be called from `MyApp::update`. Note that it doesn't get called
    /// from `MyApp::save` since that would only let it happen when autosave happens.
    fn maybe_backup_save_file(&mut self) {
        // Return early on Web.
        #[cfg(target_arch = "wasm32")]
        return;

        // Explicitly specify not wasm32 to satisfy `eframe::storage_dir`.
        #[cfg(not(target_arch = "wasm32"))]
        if self.state.tick.ticks_processed_total - self.state.session.latest_backup_tick
            >= crate::BACKUP_SAVE_TICK_INTERVAL
        {
            use eframe::storage_dir;
            if let Some(save_dir) = storage_dir(crate::SAVE_DIR) {
                // Set the latest backup tick regardless of whether backups succeeded or failed.
                // If it failed, trying again in the next tick would probably also fail, and would
                // get into a bad state of repeatedly trying to write to disk.
                self.state.session.latest_backup_tick = self.state.tick.ticks_processed_total;

                // Create backup with timestamp prefix
                let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
                let backup_name = format!("{}_{}.bak", timestamp, crate::SAVE_FILE);
                let save_path = save_dir.join(crate::SAVE_FILE);
                let backup_path = save_path.with_file_name(backup_name);

                match std::fs::copy(save_path.clone(), backup_path.clone()) {
                    Ok(_) => {
                        self.state.logger.log_debug(
                            [LogCategory::Debug],
                            format!(
                                "Completed savegame backup from {save_path:?} to {backup_path:?}",
                            ),
                        );
                    }
                    Err(error) => self.state.logger.log_error(
                        [LogCategory::Debug],
                        format!(
                            "Attempted to back up savegame from {save_path:?} to {backup_path:?} but got error: {error}",
                        ),
                    ),
                }
            }
        }
    }
}

/// https://docs.rs/eframe/latest/eframe/trait.App.html
impl eframe::App for MyApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.maybe_backup_save_file();
        self.handle_keyboard_shortcuts(ctx);

        // Old style menu bar. Has File, Settings, About, etc.
        let mut commands = self.menu_bar(ctx);

        // egui has panels like `SidePanel`, `TopBottomPanel`, `CentralPanel` which are nice for
        // simple UIs. The UIs here got too complex for it, and layout was becoming a pain
        // so I switched to egui_dock. It puts all of the content within CentralPanel. Docking
        // allows tabs to be moved around somewhat arbitrarily.
        // This lets users customize their interface.
        // * keep track of all open tabs in `self.open_tabs`
        // * tab and window locations are stored within `self.dock_state`. Outside of initialization
        //   this shouldn't need to be modified.

        // Pre-compute available tabs before creating the TabViewer.
        // Because everything is within MyApp, we cannot provide `self.dock` into DockArea::new
        // and also provide it into tab_viewer without being blocked by Rust's borrow checker.
        // Making a copy of relevant information within DockState is necessary.
        let available_tabs = get_closed_tabs(&self.dock_state);

        // This option isn't required, but I like removing the add tab interactions
        // when there aren't actually any tabs available. This assumes that all of your tabs
        // are unique: there should never be multiple of the same tab open. This may not
        // be the case for your program!
        // This boolean must be calculated before tab_viewer is created to prevent borrow issues with `self.state`.
        let show_add_buttons = !available_tabs.is_empty() && self.state.ui.dock_settings.show_add_buttons;

        // Clone to avoid borrow issues.
        let dock_settings = self.state.ui.dock_settings.clone();

        let mut style = egui_dock::Style::from_egui(ctx.style().as_ref());

        // Remove rounded corners. There is a minor graphical annoyance with them, where the
        // negative space they expose shows up as black when in light mode. It's ugly.
        style.tab_bar.corner_radius = CornerRadius::ZERO;
        style.tab.tab_body.corner_radius = CornerRadius::ZERO;
        style.tab.active.corner_radius = CornerRadius::ZERO;
        style.tab.inactive.corner_radius = CornerRadius::ZERO;
        style.tab.focused.corner_radius = CornerRadius::ZERO;
        style.tab.hovered.corner_radius = CornerRadius::ZERO;
        style.main_surface_border_rounding = CornerRadius::ZERO;

        style.overlay.overlay_type = dock_settings.overlay_type;
        style.tab_bar.fill_tab_bar = dock_settings.spaced_tabs;
        if self.state.ui.dock_settings.show_top_bar {
            // Default height as defined in: https://github.com/Adanos020/egui_dock/blob/469dd3852b65d8106df88abbef9cccdfad25a2e6/src/style.rs#L417
            style.tab_bar.height = 24.0;
        } else {
            style.tab_bar.height = 0.0;
        }

        let mut tab_viewer_commands = Vec::new();
        let mut tab_viewer = MyAppTabViewer {
            state: &mut self.state,
            available_tabs,
            commands: &mut tab_viewer_commands,
        };

        egui_dock::DockArea::new(&mut self.dock_state)
            .style(style)
            // Context menus don't seem too useful for these menus, but it's worth considering.
            .tab_context_menus(false)
            .show_close_buttons(dock_settings.show_close_buttons)
            .show_leaf_close_all_buttons(dock_settings.show_leaf_close_all)
            .show_add_buttons(show_add_buttons)
            .show_add_popup(show_add_buttons) // The popup that appears when clicking the add button.
            .show_leaf_collapse_buttons(dock_settings.show_leaf_collapse)
            .show(ctx, &mut tab_viewer);
        commands.extend(tab_viewer_commands);
        self.state.process_commands(ctx, &mut self.dock_state, &commands);

        // Always request repainting to ensure that the UI stays up to date.
        // Without this, it only updates the UI if the mouse moves.
        ctx.request_repaint();
        self.state.update(ctx);
    }

    /// Called on shutdown, and at regular intervals (based on [`Self::auto_save_interval`].
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        match self.save_to_storage(storage) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to save: {e}");
                self.state
                    .logger
                    .log_error([LogCategory::Debug], format!("Failed to save: {e:?}"));
                self.state.session.save.latest_save_error = Some(e);
            }
        }
    }
    /// Time between automatic calls to Self::save
    fn auto_save_interval(&self) -> std::time::Duration {
        // This is a hack to allow triggering a save from the UI.
        // https://github.com/emilk/egui/issues/5243
        if self.state.session.save.needs_save {
            std::time::Duration::from_secs(0)
        } else {
            std::time::Duration::from_secs(15)
        }
    }
    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut RawInput) {
        /*
        A hook for manipulating or filtering raw input before it is processed by Self::update.
        This function provides a way to modify or filter input events before they are processed by egui.
        It can be used to prevent specific keyboard shortcuts or mouse events from being processed by egui.
        Additionally, it can be used to inject custom keyboard or mouse events into the input stream, which can be useful for implementing features like a virtual keyboard.
        */
    }
}

impl MyAppState {
    const MAX_TICKS_PER_UPDATE: i32 = 50;

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Note that this is related to game ticks, but is not the same. It is called `update`
    /// for consistency with `eframe::App::update`, which calls this method.
    ///
    /// Ticks need to be separate from UI updates because otherwise the game would run
    /// faster for people with better computers.
    ///
    /// Game ticks are executed from within this method (see `MyApp::tick()`) but the method may be
    /// called 0 to MAX_TICKS_PER_UPDATE times, where the upper limit is to prevent noticeable UI lag.
    pub fn update(&mut self, ctx: &egui::Context) {
        let current_time = chrono::Local::now();
        // Cap the delta time to account for the game being saved and loaded, or system time being
        // changed intentionally.
        let delta_time = current_time
            .signed_duration_since(self.tick.last_update_time)
            .min(Duration::milliseconds(100))
            .max(Duration::default());
        self.tick.last_update_time = current_time;
        self.tick.accumulated_delta_time += delta_time;
        self.tick.time_elapsed_total += delta_time;

        let mut ticks_to_process: i32 = 0;
        if self.tick.ticks_active {
            while self.tick.accumulated_delta_time >= self.tick.target_tick_interval {
                ticks_to_process += 1;
                self.tick.accumulated_delta_time -= self.tick.target_tick_interval;

                // Safety cap: if the accumulated time is growing faster than we can process game
                //  ticks, declare bankruptcy on the extra ones.
                if ticks_to_process >= Self::MAX_TICKS_PER_UPDATE {
                    self.tick.accumulated_delta_time = Duration::zero();
                    break;
                }
            }
        }
        for _ in 0..ticks_to_process {
            self.tick();
        }
        self.tick.ticks_processed_total += ticks_to_process as u128;
        self.tick.ticks_processed_current_loop += ticks_to_process as u128;

        // Always request repainting to ensure that the UI stays up to date.
        // Without this, it only updates the UI if the mouse moves.
        ctx.request_repaint();

        self.session.fps_counter.update();
    }
}
