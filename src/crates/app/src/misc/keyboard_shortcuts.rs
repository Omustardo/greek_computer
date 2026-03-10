use crate::{MyApp, MyAppState};
use std::collections::HashMap;
use std::sync::LazyLock;

/// To add a new keyboard shortcut:
/// 1. Add an entry in SHORTCUTS. It is allowed for multiple keybinds to map to the same `KeyAction`.
/// 2. Add an implementation in `KeyAction::execute`.
///
/// These are all global shortcuts. They will be handled from any UI.
/// If you want to add a shortcut within a specific UI, do it within that UI's implementation.
/// But be careful since it will be hard for users to manage these, and it risks conflict with top
/// level shortcuts here.
//
// Single source of truth for all shortcuts
static SHORTCUTS: LazyLock<Vec<(KeyCombo, KeyAction, &'static str)>> = LazyLock::new(|| {
    let mut shortcuts = vec![
        #[cfg(not(target_arch = "wasm32"))]
        (
            KeyCombo::new(egui::Key::Q).ctrl(),
            KeyAction::ExitApplication,
            "Exit application",
        ),
        #[cfg(not(target_arch = "wasm32"))]
        (
            KeyCombo::new(egui::Key::F11),
            KeyAction::ToggleFullscreen,
            "Toggle fullscreen",
        ),
        (
            KeyCombo::new(egui::Key::F).ctrl(),
            KeyAction::FocusConsoleInput,
            "Focus cursor on log input",
        ),
        // Shortcuts built into egui. They are included here to generate documentation alongside the other shortcuts.
        (
            KeyCombo::new(egui::Key::Escape),
            KeyAction::RemoveUIFocus,
            "Remove current UI focus",
        ),
        (
            KeyCombo::new(egui::Key::Tab),
            KeyAction::SwitchUIFocus,
            "Switch UI focus",
        ),
        (
            KeyCombo::new(egui::Key::Plus).ctrl(), // technically this is handled by the browser on web, but it should always work.
            KeyAction::ZoomFactor,
            "Zoom in / out",
        ),
    ];

    // Sort once at initialization time
    shortcuts.sort_by_key(|(combo, _, _)| *combo);
    // DEBUG:
    // for (combo, action, _) in shortcuts.iter() {
    //     println!("{:?} {:?}", combo, action);
    // }

    assert_no_overlapping_shortcuts(&shortcuts);

    shortcuts
});

// Things that aren't technically key shortcuts, but still are worth listing.
// If this grows, KeyCombo should be renamed to `InputCombo` and it should be an enum containing
// `KeyCombo`, `ScrollCombo`, and any other potential inputs.
static FAKE_SHORTCUTS: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| vec![("Ctrl + MouseWheel", "Zoom in / out")]);

// Action enum that represents all possible keyboard actions
#[derive(Debug, Clone)]
enum KeyAction {
    #[cfg(not(target_arch = "wasm32"))]
    ExitApplication,
    #[cfg(not(target_arch = "wasm32"))]
    ToggleFullscreen,

    FocusConsoleInput,

    // Built-in actions. Handled by egui, but included them for documentation purposes.
    RemoveUIFocus,
    SwitchUIFocus,
    ZoomFactor,
}

impl KeyAction {
    pub fn execute(&self, app: &mut MyApp, ctx: &egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            // ctx is unused on web
            _ = ctx;
        }

        match self {
            #[cfg(not(target_arch = "wasm32"))]
            KeyAction::ExitApplication => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            #[cfg(not(target_arch = "wasm32"))]
            KeyAction::ToggleFullscreen => {
                app.state.ui.is_fullscreen = !app.state.ui.is_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.state.ui.is_fullscreen));
            }
            KeyAction::FocusConsoleInput => {
                if app.state.logger.show_input_area {
                    app.state.logger.should_focus_input = true;
                }
            }
            // These are handled by egui itself, so no action needed
            KeyAction::RemoveUIFocus | KeyAction::SwitchUIFocus | KeyAction::ZoomFactor => {}
        }
    }
}

impl MyApp {
    pub fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.options_mut(|opts| {
            // This is egui's default setting, but explicitly set keyboard zoom (`ctrl + +` and `ctrl + -`).
            // Just in case the default changes.
            // It should be true on Desktop and False on web, so that browsers can handle it themselves.
            opts.zoom_with_keyboard = cfg!(not(target_arch = "wasm32"));
        });

        self.handle_mouse_zoom(ctx);

        let input = ctx.input(Clone::clone);

        // Check all shortcuts and execute matching ones
        for (combo, action, _) in SHORTCUTS.iter() {
            if combo.matches(&input) {
                action.execute(self, ctx);
                // Note: We don't break here in case multiple shortcuts could match,
                // though in practice this shouldn't happen. There is an assertion to prevent it.
            }
        }
    }

    /// Handle zooming in and out using while holding ctrl and scrolling.
    fn handle_mouse_zoom(&mut self, ctx: &egui::Context) {
        let zoom_delta = ctx.input_mut(|input| {
            // `input.smooth_scroll_delta.y` didn't work in my experience and I'm not sure why.
            // Raw seems fine since we only care about it being negative or positive, not the value.
            let scroll_delta = input.raw_scroll_delta.y;
            if input.modifiers.ctrl && scroll_delta != 0.0 {
                // Consume the scroll event to prevent other handlers from using it.
                input.smooth_scroll_delta.y = 0.0;

                // Use steps rather than using scroll delta directly since the units are different.
                const ZOOM_STEP: f32 = 0.1;
                if scroll_delta > 0.0 { ZOOM_STEP } else { -ZOOM_STEP }
            } else {
                0.0
            }
        });
        if zoom_delta != 0.0 {
            let new_zoom_factor = ctx.zoom_factor() + zoom_delta;
            const MIN_ZOOM: f32 = 0.5;
            const MAX_ZOOM: f32 = 3.0;
            ctx.set_zoom_factor(new_zoom_factor.clamp(MIN_ZOOM, MAX_ZOOM));
        }
    }
}

impl MyAppState {
    /// Show the controls menu button with all available shortcuts
    pub fn show_controls_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Controls", |ui| {
            // Shortcuts are already sorted, no need to sort again
            for (combo, _, description) in SHORTCUTS.iter() {
                self.show_shortcut_row(ui, combo, description);
            }
            for (combo_text, description) in FAKE_SHORTCUTS.iter() {
                self.show_shortcut_row_text(ui, combo_text.to_string(), description);
            }
        });
    }

    fn show_shortcut_row(&self, ui: &mut egui::Ui, combo: &KeyCombo, description: &str) {
        self.show_shortcut_row_text(ui, combo.to_display_string(), description);
    }
    fn show_shortcut_row_text(&self, ui: &mut egui::Ui, combo_text: String, description: &str) {
        ui.horizontal(|ui| {
            // Show key combination in monospace font
            ui.add_sized(
                [80.0, 20.0],
                egui::Label::new(
                    egui::RichText::new(combo_text)
                        .monospace()
                        .color(ui.visuals().strong_text_color()),
                ),
            );
            ui.label(description);
        });
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct KeyCombo {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: egui::Key,
}

#[allow(dead_code)]
impl KeyCombo {
    pub fn new(key: egui::Key) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn matches(&self, input: &egui::InputState) -> bool {
        input.key_pressed(self.key)
            && input.modifiers.ctrl == self.ctrl
            && input.modifiers.shift == self.shift
            && input.modifiers.alt == self.alt
    }

    /// Checks if this key combination is a subset of another.
    /// This is true if they share the same key and all of `self`'s active
    /// modifiers are also active in `other`. This is the condition for an overlap.
    fn is_subset_of(&self, other: &Self) -> bool {
        if self.key != other.key {
            return false;
        }
        if self.ctrl && !other.ctrl {
            return false;
        }
        if self.alt && !other.alt {
            return false;
        }
        if self.shift && !other.shift {
            return false;
        }
        true
    }

    pub fn to_display_string(self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.ctrl && self.key == egui::Key::Plus {
            return "Ctrl +/-".to_string();
        }

        parts.push(self.key.name());
        parts.join("+")
    }
}

/// Panics if there are any overlapping keybinds. For example, CTRL+F and CTRL+SHIFT+F.
fn assert_no_overlapping_shortcuts(shortcuts: &[(KeyCombo, KeyAction, &'static str)]) {
    // This function groups shortcuts by their base `egui::Key` and then checks for
    // modifier subset overlaps only within those small groups. This avoids an O(n^2)
    // complexity of comparing every shortcut with every other shortcut.

    // Group shortcuts by the non-modifier key.
    let mut key_map: HashMap<egui::Key, Vec<(KeyCombo, &'static str)>> = HashMap::new();
    for (combo, _, description) in shortcuts {
        key_map.entry(combo.key).or_default().push((*combo, *description));
    }

    // For each key, check for overlapping modifier combinations.
    for shortcuts_for_key in key_map.values() {
        // If there's only one shortcut for this key, no overlap is possible.
        if shortcuts_for_key.len() <= 1 {
            continue;
        }

        // Now, perform a O(n^2) check, but only on the very small group
        // of shortcuts that share the same base key.
        for (i, (combo1, desc1)) in shortcuts_for_key.iter().enumerate() {
            for (j, (combo2, desc2)) in shortcuts_for_key.iter().enumerate() {
                if i == j {
                    continue;
                }

                assert!(
                    !combo1.is_subset_of(combo2),
                    "Overlapping keybinds detected for key '{}'!\n\
                         - Keybind 1: '{}' ({})\n\
                         - Keybind 2: '{}' ({})\n\
                         The first keybind is completely shadowed by the second one.\n\
                         This needs to be fixed in code at keyboard_shortcuts.rs.",
                    combo1.key.name(),
                    combo1.to_display_string(),
                    desc1,
                    combo2.to_display_string(),
                    desc2
                );
            }
        }
    }
}

#[cfg(test)]
mod optimized_tests {
    use super::*;

    #[test]
    fn test_global_shortcuts_are_valid() {
        assert_no_overlapping_shortcuts(&SHORTCUTS);
    }

    #[test]
    fn test_optimized_no_overlaps() {
        let shortcuts = vec![
            (KeyCombo::new(egui::Key::A).ctrl(), KeyAction::ExitApplication, "Ctrl+A"),
            (
                KeyCombo::new(egui::Key::A).shift(),
                KeyAction::ExitApplication,
                "Shift+A",
            ),
            (KeyCombo::new(egui::Key::B).ctrl(), KeyAction::ExitApplication, "Ctrl+B"),
        ];
        assert_no_overlapping_shortcuts(&shortcuts);
    }

    #[test]
    #[should_panic(expected = "Overlapping keybinds detected for key 'A'!")]
    fn test_optimized_identical_shortcuts_should_panic() {
        let shortcuts = vec![
            (
                KeyCombo::new(egui::Key::A).ctrl(),
                KeyAction::FocusConsoleInput,
                "Action 1",
            ),
            (
                KeyCombo::new(egui::Key::B).shift(),
                KeyAction::FocusConsoleInput,
                "Different Key",
            ),
            (
                KeyCombo::new(egui::Key::A).ctrl(),
                KeyAction::FocusConsoleInput,
                "Action 2",
            ),
        ];
        assert_no_overlapping_shortcuts(&shortcuts);
    }

    #[test]
    #[should_panic(expected = "Overlapping keybinds detected for key 'C'!")]
    fn test_optimized_subset_shortcuts_should_panic() {
        let shortcuts = vec![
            (
                KeyCombo::new(egui::Key::C).ctrl(),
                KeyAction::FocusConsoleInput,
                "Subset action",
            ),
            (
                KeyCombo::new(egui::Key::D),
                KeyAction::FocusConsoleInput,
                "Some other key",
            ),
            (
                KeyCombo::new(egui::Key::C).ctrl().shift(),
                KeyAction::FocusConsoleInput,
                "Superset action",
            ),
        ];
        assert_no_overlapping_shortcuts(&shortcuts);
    }

    #[test]
    #[should_panic(expected = "Overlapping keybinds detected for key 'C'!")]
    fn test_optimized_subset_in_reverse_order_should_panic() {
        let shortcuts = vec![
            (
                KeyCombo::new(egui::Key::C).ctrl().shift(),
                KeyAction::FocusConsoleInput,
                "Superset action",
            ),
            (
                KeyCombo::new(egui::Key::C).ctrl(),
                KeyAction::FocusConsoleInput,
                "Subset action",
            ),
        ];
        assert_no_overlapping_shortcuts(&shortcuts);
    }

    #[test]
    fn test_optimized_different_keys_with_same_modifiers() {
        let shortcuts = vec![
            (KeyCombo::new(egui::Key::X).ctrl(), KeyAction::ExitApplication, "Ctrl+X"),
            (KeyCombo::new(egui::Key::Y).ctrl(), KeyAction::ExitApplication, "Ctrl+Y"),
        ];
        assert_no_overlapping_shortcuts(&shortcuts);
    }
}
