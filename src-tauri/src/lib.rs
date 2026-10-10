mod auth;
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
mod settings;
mod state;
mod tasks;

use auth::endpoints::Endpoints;
use auth::store::{AccountStore, KeyringStore};
use auth::AuthService;
use paths::DataPaths;
use std::sync::Arc;
use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let paths = DataPaths::new(app.path().app_data_dir()?);
            let client = http::build_client()?;
            let accounts = AccountStore::new(paths.accounts_file(), Arc::new(KeyringStore));
            let auth = AuthService::new(client.clone(), Endpoints::default(), accounts);
            app.manage(AppState::new(client, paths, auth));
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
            commands::get_accounts,
            commands::sign_in,
            commands::cancel_sign_in,
            commands::set_active_account,
            commands::remove_account,
            commands::get_auth_config,
            commands::set_ms_client_id,
            commands::open_login_page,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Zenith");
}
