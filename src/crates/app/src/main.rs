#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser)]
#[command(name = "my-app")] // TEMPLATE_TODO: change the app name
#[command(about = "An app")] // TEMPLATE_TODO: change the "about"
struct Args {
    /// Allow multiple instances of the application to run simultaneously. Off by default to prevent
    /// multiple instances from overwriting each other's savedata.
    #[arg(long)]
    allow_multiple_instances: bool,
}

#[cfg(not(target_arch = "wasm32"))]
use fs2::FileExt;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::OpenOptions;
#[cfg(not(target_arch = "wasm32"))]
use std::io;

#[cfg(not(target_arch = "wasm32"))]
struct InstanceGuard {
    _file: std::fs::File,
}

#[cfg(not(target_arch = "wasm32"))]
impl InstanceGuard {
    fn new(lock_path: std::path::PathBuf) -> Result<Self, io::Error> {
        // Ensure the directory exists
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(lock_path)?;

        // Try to acquire exclusive lock (non-blocking)
        file.try_lock_exclusive()?;

        Ok(InstanceGuard { _file: file })
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use eframe::storage_dir;

    let args = Args::parse();

    if !args.allow_multiple_instances {
        // Only allow one instance of the app to run
        // This is implemented through a lock file in the same directory as the save data.
        let lock_path = storage_dir(app::SAVE_DIR)
            .map(|path| path.join("app.lock"))
            .unwrap_or_else(|| std::path::PathBuf::from("app.lock"));
        let _instance_guard = match InstanceGuard::new(lock_path) {
            Ok(guard) => guard,
            Err(_) => {
                eprintln!(
                    "Another instance of the app is already running! Exiting to prevent multiple apps from overwriting the same save data. You can disable this check with --allow_multiple_instances"
                );
                std::process::exit(1);
            }
        };
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(false)
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        persistence_path: storage_dir(app::SAVE_DIR)
            .map(|path| path.join(app::SAVE_FILE))
            .or(None),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        // TEMPLATE_TODO: Change the app_name.
        "My App",
        native_options,
        Box::new(|cc| Ok(Box::new(app::MyApp::new(cc)))),
    )
}

// When compiling to web:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window").document().expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|cc| Ok(Box::new(app::MyApp::new(cc)))))
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html("<p> The app has crashed. See the developer console for details. </p>");
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
