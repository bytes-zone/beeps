mod app;
mod schema;
mod tables;
mod ui_document;

use app::App;
use specta_typescript::Typescript;
use tauri::{async_runtime::Mutex, AppHandle, Manager};
use tauri_specta::{collect_commands, Builder};
use ui_document::{PingWithTag, UiDocument};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![document]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../ui/src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.manage(Mutex::new(App::load(&app::database_url())?));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![document])
        .invoke_handler(tauri::generate_handler![schedule_pings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Get the current document.
#[tauri::command]
#[specta::specta]
async fn document(app: AppHandle) -> UiDocument {
    let lock = app.state::<Mutex<App>>();
    let app = lock.lock().await;

    app.document().into()
}

/// Advance our timeline of pings to the present. This will insert new pings
/// into the document, and return them to the frontend. As a reminder, this
/// returns one ping into the future. Don't show that one to the user!
#[tauri::command]
#[specta::specta]
async fn schedule_pings(app: AppHandle) -> Result<Vec<PingWithTag>, String> {
    let lock = app.state::<Mutex<App>>();
    let mut app = lock.lock().await;

    match app.schedule_pings() {
        Ok(new_pings) => Ok(new_pings.into_iter().map(PingWithTag::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}
