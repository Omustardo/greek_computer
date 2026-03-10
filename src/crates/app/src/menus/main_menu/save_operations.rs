use crate::MyApp;
use crate::log_categories::LogCategory::{Command, Debug};
use base64::Engine;
use base64::engine::general_purpose;
use std::collections::HashMap;
// WARNING: Saving and loading is a bit complex.
// The full save file is RON. RON is basically json but with Rust syntax.
// The save file is handled by eframe, and is a key-value mapping.
// All of `MyApp` is serialized into the "app" key.
// There's also data from egui in the "egui" key, and some window size information in "window".
//
// There is additional complexity to support the various options:
// * Saving to file on desktop (handled through eframe's [`MyApp::save()`])
// * Export to file on web
// * Export to clipboard and import from text
//
// Just remember that all of the save formats are RON serialiaztion of a `HashMap<String,String>`.
// The most important key in the map is [`eframe::APP_KEY`] which maps to a serialized and then
// base64 encoded `MyApp`.

impl MyApp {
    /// Load the saved app state (if any).
    ///
    /// The state is loaded from persistent storage, so this requires the `persistence` feature
    /// enabled in Cargo.toml for both egui and eframe.
    #[must_use]
    pub fn load_from_storage(storage: &dyn eframe::Storage) -> Option<MyApp> {
        if let Some(encoded_data) = storage.get_string(eframe::APP_KEY) {
            Self::deserialize_app(&encoded_data).ok()
        } else {
            None
        }
    }

    // TODO(P1): Apply compression to the serialized MyApp.
    // TODO(P2): Obfuscate save data.
    fn serialize_app(&self) -> Result<String, String> {
        // Dereference and take a reference to avoid borrow issues. `encode_to_vec` doesn't need
        // to mutate, so if we dereference the `&mut self` and only provide a reference, it lets
        // `self` be mutated later on. Specifically so that we can call `self.state.logger.log_`
        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| format!("Failed to serialize to bincode: {e}"))?;

        Ok(general_purpose::STANDARD.encode(&bytes))
    }
    /// WARNING: This only deserializes MyApp. This is not the same as a full save file!
    /// See the documentation elsewhere in this file for what else is included in a save file.
    fn deserialize_app(encoded_data: &str) -> Result<MyApp, String> {
        // Decode from base64
        let bytes = general_purpose::STANDARD
            .decode(encoded_data)
            .map_err(|e| format!("Failed to decode base64: {e}"))?;

        // Deserialize from bincode
        let config = bincode::config::standard();
        let (app, _) = bincode::serde::decode_from_slice::<MyApp, _>(&bytes, config)
            .map_err(|e| format!("Failed to deserialize bincode data: {e}"))?;

        Ok(app)
    }

    /// Saves the app state to file.
    ///
    /// Requires the egui “persistence” feature to be enabled in both egui and eframe.
    /// On web the state is stored to “Local Storage”.
    /// On native the path is set in [`NativeOptions::persistence_path`].
    /// See [`MyApp::render_file_menu`] for other save-related code.
    ///
    /// Note that storage goes to a single file at the persistence_path location. The keys
    /// and values are all within this file, similar to json. It includes keys not set here,
    /// like egui's internal state.
    pub fn save_to_storage(&mut self, storage: &mut dyn eframe::Storage) -> Result<(), String> {
        // Set these values before saving to avoid potential weirdness when loading the save file later on.
        // For example, if needs_save were true, it would try to save immediately upon loading a game file.
        // It wouldn't be a loop or anything, just odd.
        self.state.session.save.needs_save = false;
        let tmp = self.state.session.save.latest_save_time;
        self.state.session.save.latest_save_time = chrono::Local::now();
        self.state.session.save.latest_save_error = None;

        match self.serialize_app() {
            Err(err) => {
                // Set the latest save time back, since it failed.
                self.state.session.save.latest_save_time = tmp;
                self.state.session.save.latest_save_error = Some(err.clone());
                Err(err)
            }
            Ok(encoded) => {
                // WARNING: I don't like that this has no error return. It's certainly possible
                // for saving to fail at this step, and my `latest_save_time` would then be wrong.
                // I don't see any way to fix it, and it should nearly impossible to hit the error.
                storage.set_string(eframe::APP_KEY, encoded);
                Ok(())
            }
        }
    }

    /// Export the complete eframe storage (including MyApp + egui state) to clipboard
    pub fn export_save_to_clipboard(&mut self, ctx: &egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            self.export_save_to_clipboard_web(ctx);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.export_save_to_clipboard_desktop(ctx);
        }
    }

    /// Web implementation: Export all localStorage as RON key-value pairs
    #[cfg(target_arch = "wasm32")]
    fn export_save_to_clipboard_web(&mut self, ctx: &egui::Context) {
        match self.read_web_storage() {
            Ok(storage_map) => match ron::to_string(&storage_map) {
                Ok(save_data) => {
                    ctx.copy_text(save_data);
                    self.state
                        .logger
                        .log_info([Command], "Save data copied to clipboard!".to_string());
                }
                Err(e) => {
                    self.state
                        .logger
                        .log_error([Command, Debug], format!("Failed to serialize save data: {e}"));
                }
            },
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to read web storage: {e}"));
            }
        }
    }

    /// Desktop implementation
    #[cfg(not(target_arch = "wasm32"))]
    fn export_save_to_clipboard_desktop(&mut self, ctx: &egui::Context) {
        // Create a storage map with current state (like web does)
        let mut storage_map = HashMap::new();

        // Add current app state.
        match self.serialize_app() {
            Ok(app_data) => {
                storage_map.insert(eframe::APP_KEY.to_string(), app_data);
            }
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to serialize current app state: {e}"));
                return;
            }
        }
        // Add current egui memory state
        let current_memory = ctx.memory(|mem| mem.clone());
        match ron::to_string(&current_memory) {
            Ok(memory_data) => {
                storage_map.insert("egui".to_string(), memory_data);
            }
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to serialize egui memory: {e}"));
            }
        }

        match ron::to_string(&storage_map) {
            Ok(save_data) => {
                ctx.copy_text(save_data);
                self.state
                    .logger
                    .log_info([Command], "Save data copied to clipboard!".to_string());
            }
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to export save data: {e}"));
            }
        }
    }

    /// Export save to file (web only)
    /// This triggers a download of the file in the web browser.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn export_save_to_file(&mut self) {
        match self.read_web_storage() {
            Ok(storage_map) => match ron::to_string(&storage_map) {
                Ok(save_data) => {
                    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
                    // TEMPLATE_TODO: Change my_app to your app_name.
                    let filename = format!("{}-my_app.ron", timestamp);

                    if let Err(e) = self.download_file(&save_data, &filename) {
                        self.state
                            .logger
                            .log_error([Command], format!("Failed to download save file: {:?}", e));
                    } else {
                        self.state
                            .logger
                            .log_info([Command], format!("Save file download triggered: {}", filename));
                    }
                }
                Err(e) => {
                    self.state
                        .logger
                        .log_error([Command], format!("Failed to serialize save data: {}", e));
                }
            },
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to read web storage: {e}"));
            }
        }
    }

    /// Import save data from text and apply it
    pub(crate) fn import_save_from_text(&mut self, ctx: &egui::Context) {
        if self.state.session.import_text_buffer.trim().is_empty() {
            self.state
                .logger
                .log_error([Command, Debug], "No save data provided".to_string());
            return;
        }

        match self.deserialize_and_apply_save(&self.state.session.import_text_buffer.clone(), ctx) {
            Ok(()) => {
                self.state
                    .logger
                    .log_info([Command], "Save file imported successfully!".to_string());
                self.state.session.import_text_buffer.clear();
            }
            Err(e) => {
                self.state
                    .logger
                    .log_error([Command, Debug], format!("Failed to import save: {e}"));
            }
        }
    }

    /// Parse and apply save data to current app state
    /// Note that this doesn't necessarily apply all of the keys from the eframe save file.
    /// It only applies keys that are explicitly implemented: app, egui
    /// As of 2025-09, this excludes "window" because modifying the window size from within
    /// a running program feels unintuitive, and it's not likely that a window manager
    /// would support it in all situations.
    fn deserialize_and_apply_save(&mut self, save_data: &str, ctx: &egui::Context) -> Result<(), String> {
        let storage_map: HashMap<String, String> =
            ron::from_str(save_data).map_err(|e| format!("Failed to parse save file: {e}"))?;

        if let Some(app_data_base64) = storage_map.get(eframe::APP_KEY) {
            let imported_app = Self::deserialize_app(app_data_base64)?;
            *self = imported_app;
        }

        // Restore egui memory state from the imported storage
        if let Some(memory_data) = storage_map.get("egui") {
            if let Ok(imported_memory) = ron::from_str::<egui::Memory>(memory_data) {
                ctx.memory_mut(|mem| *mem = imported_memory);
            } else {
                self.state.logger.log_error(
                    [Command, Debug],
                    "Failed to parse egui memory from save data".to_string(),
                );
            }
        }

        // TODO: Is this needed? We don't save desktop games to local storage
        //   immediately. Should we really do it for web? I think it's the correct behavior,
        //   but there isn't a pretty way to force `eframe` to save. https://github.com/emilk/egui/issues/5243
        #[cfg(target_arch = "wasm32")]
        {
            self.apply_storage_to_web_storage(&storage_map)?;
        }

        self.state
            .logger
            .log_info([Command], "Game loaded from save".to_string());
        Ok(())
    }

    /// Read all key-value pairs from web localStorage into a HashMap.
    ///
    /// This is used by export functions to gather all persistent storage data
    /// (app state, egui memory, window settings) for export to clipboard or file.
    #[cfg(target_arch = "wasm32")]
    fn read_web_storage(&self) -> Result<HashMap<String, String>, String> {
        let window = web_sys::window().ok_or("Failed to get window")?;
        let storage = window
            .local_storage()
            .map_err(|_| "Failed to get localStorage")?
            .ok_or("localStorage not available")?;

        let mut storage_map = HashMap::new();
        let length = storage.length().map_err(|_| "Failed to get localStorage length")?;

        for i in 0..length {
            if let Some(key) = storage.key(i).map_err(|_| "Failed to get key")? {
                if let Some(value) = storage.get_item(&key).map_err(|_| "Failed to get item")? {
                    storage_map.insert(key, value);
                }
            }
        }
        Ok(storage_map)
    }

    /// Apply storage map to web localStorage
    #[cfg(target_arch = "wasm32")]
    fn apply_storage_to_web_storage(&self, storage_map: &HashMap<String, String>) -> Result<(), String> {
        let window = web_sys::window().ok_or("Failed to get window")?;
        let storage = window
            .local_storage()
            .map_err(|_| "Failed to get localStorage")?
            .ok_or("localStorage not available")?;

        storage.clear().map_err(|_| "Failed to clear localStorage")?;

        for (key, value) in storage_map {
            storage
                .set_item(key, value)
                .map_err(|_| format!("Failed to set localStorage item: {}", key))?;
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn download_file(&self, content: &str, filename: &str) -> Result<(), wasm_bindgen::JsValue> {
        use wasm_bindgen::prelude::*;
        use web_sys::*;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let array = js_sys::Array::new();
        array.push(&JsValue::from_str(content));

        let blob = web_sys::Blob::new_with_str_sequence(&array)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let anchor = document.create_element("a")?.dyn_into::<web_sys::HtmlAnchorElement>()?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.style().set_property("display", "none")?;

        document.body().unwrap().append_child(&anchor)?;
        anchor.click();
        document.body().unwrap().remove_child(&anchor)?;

        web_sys::Url::revoke_object_url(&url)?;

        Ok(())
    }

    /// Clear all save data (memory only).
    #[cfg(debug_assertions)]
    pub(crate) fn clear_all_data(&mut self) {
        *self = Self::default();

        self.state.logger.log_info([Command], "Data cleared.".to_string());
    }
}

#[cfg(test)]
// Mock storage implementation for testing
struct MockStorage {
    data: HashMap<String, String>,
}

#[cfg(test)]
impl MockStorage {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }
}

#[cfg(test)]
impl eframe::Storage for MockStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }

    fn flush(&mut self) {
        // No-op for testing
    }
}

#[cfg(test)]
mod save_load_tests {
    use super::*;
    use crate::MyApp;
    use eframe::Storage;
    use std::collections::HashMap;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original_app = MyApp::default();

        // Test serialization
        let encoded = original_app.serialize_app().expect("Failed to serialize");
        assert!(!encoded.is_empty());

        // Test deserialization
        let decoded_app = MyApp::deserialize_app(&encoded).expect("Failed to deserialize");

        // Basic sanity checks
        assert_eq!(
            original_app.state.ui.current_layout_name,
            decoded_app.state.ui.current_layout_name,
        );
        assert_eq!(
            original_app.state.tick.ticks_active,
            decoded_app.state.tick.ticks_active,
        );
    }

    #[test]
    fn test_save_load_storage() {
        let mut original_app = MyApp::default();

        // Modify some state to make the test meaningful
        original_app.state.tick.ticks_active = false;
        original_app.state.tick.ticks_processed_total = 12345;

        let mut storage = MockStorage::new();

        // Save to storage
        original_app.save_to_storage(&mut storage).expect("Failed to save");

        // Load from storage
        let loaded_app = MyApp::load_from_storage(&storage).expect("Failed to load");

        // Verify the data matches
        assert_eq!(
            original_app.state.tick.ticks_processed_total,
            loaded_app.state.tick.ticks_processed_total
        );
    }

    #[test]
    fn test_export_import_format() {
        let mut original_app = MyApp::default();
        original_app.state.tick.ticks_processed_total = 9999;

        // Create a mock storage with app data and some egui data
        let mut storage = MockStorage::new();
        original_app.save_to_storage(&mut storage).expect("Failed to save");
        storage.set_string("egui", "test_egui_data".to_string());
        storage.set_string("window", "test_window_data".to_string());

        // Simulate export format (RON HashMap)
        let export_data = ron::to_string(&storage.data).expect("Failed to create export data");

        // Test that we can parse it back
        let parsed_map: HashMap<String, String> = ron::from_str(&export_data).expect("Failed to parse export data");

        // Verify the app data can be deserialized
        let app_data = parsed_map.get(eframe::APP_KEY).expect("Missing app data");
        let imported_app = MyApp::deserialize_app(app_data).expect("Failed to deserialize app");

        assert_eq!(
            original_app.state.tick.ticks_processed_total,
            imported_app.state.tick.ticks_processed_total
        );
    }

    #[test]
    fn test_invalid_base64_handling() {
        let result = MyApp::deserialize_app("invalid_base64!");
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Failed to decode base64"));
    }

    #[test]
    fn test_invalid_bincode_handling() {
        // Valid base64 but invalid bincode data
        let invalid_bincode = base64::engine::general_purpose::STANDARD.encode(b"not bincode data");
        let result = MyApp::deserialize_app(&invalid_bincode);
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Failed to deserialize bincode"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Helper to create test save data that simulates cross-platform exports
    fn create_test_export_data(app: &MyApp) -> String {
        let mut storage_map = HashMap::new();

        // Add app data (bincode)
        let app_data = app.serialize_app().expect("Failed to serialize app");
        storage_map.insert(eframe::APP_KEY.to_string(), app_data);

        // Add mock egui data (RON)
        storage_map.insert("egui".to_string(), "test_egui_memory".to_string());

        // Add mock window data (RON)
        storage_map.insert("window".to_string(), "test_window_config".to_string());

        ron::to_string(&storage_map).expect("Failed to serialize export data")
    }

    #[test]
    fn test_web_to_desktop_export_import() {
        let mut original_app = MyApp::default();
        original_app.state.tick.ticks_processed_total = 42;

        // Simulate web export (what would be copied to clipboard)
        let export_data = create_test_export_data(&original_app);

        // Simulate desktop import
        let mut target_app = MyApp::default();

        // This tests the import logic without actually using egui context
        let storage_map: HashMap<String, String> = ron::from_str(&export_data).expect("Failed to parse export data");

        if let Some(app_data) = storage_map.get(eframe::APP_KEY) {
            let imported_app = MyApp::deserialize_app(app_data).expect("Failed to deserialize");
            target_app = imported_app;
        }

        assert_eq!(original_app.state.tick.ticks_processed_total, target_app.state.tick.ticks_processed_total);
    }

    #[test]
    fn test_desktop_to_desktop_compatibility() {
        // Test that desktop save files can be loaded on different desktop instances
        let mut original_app = MyApp::default();

        // Simulate saving to file (what eframe persistence does)
        let mut file_storage = MockStorage::new();
        original_app.save_to_storage(&mut file_storage).expect("Failed to save");

        // Simulate loading on a different desktop instance
        let _ = MyApp::load_from_storage(&file_storage).expect("Failed to load");
    }
}
