mod commands;
mod download;
mod error;
mod http;
mod install;
mod instances;
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            commands::launch_instance,
            commands::stop_game,
            commands::list_instances,
            commands::get_instance,
            commands::create_instance,
            commands::update_instance,
            commands::set_instance_icon,
            commands::get_instance_icon,
            commands::duplicate_instance,
            commands::delete_instance,
            commands::open_instance_folder,
            commands::pick_icon_file,
            commands::pick_java_file,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Zenith");
}
