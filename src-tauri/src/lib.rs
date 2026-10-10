mod commands;
mod download;
mod error;
mod http;
mod install;
mod java;
mod launch;
mod mojang;
mod paths;
mod state;

use paths::DataPaths;
use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = DataPaths::new(app.path().app_data_dir()?);
            let client = http::build_client()?;
            app.manage(AppState::new(client, paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_version_manifest,
            commands::list_installed_versions,
            commands::install_version,
            commands::cancel_install,
            commands::launch_game,
            commands::stop_game,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Zenith");
}
