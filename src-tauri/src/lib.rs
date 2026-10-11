mod app_id;
mod auth;
mod commands;
mod download;
mod error;
mod http;
mod install;
mod instances;
mod java;
mod launch;
mod loaders;
mod modrinth;
mod mojang;
mod paths;
mod skins;
mod state;
mod tasks;
#[cfg(test)]
mod test_support;

use auth::endpoints::Endpoints;
use auth::store::{AccountStore, KeyringStore};
use auth::AuthService;
use modrinth::api::ModrinthApi;
use skins::api::SkinsApi;
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
            let modrinth = ModrinthApi::new(client.clone());
            let skins = SkinsApi::new(client.clone());
            app.manage(AppState::new(client, paths, auth, modrinth, skins));
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
            commands::list_loader_versions,
            commands::modrinth_search,
            commands::modrinth_project,
            commands::modrinth_versions,
            commands::modrinth_categories,
            commands::list_content,
            commands::set_content_enabled,
            commands::remove_content,
            commands::install_content,
            commands::check_content_updates,
            commands::update_content,
            commands::install_modpack,
            commands::import_mrpack,
            commands::export_mrpack,
            commands::pick_mrpack_file,
            commands::pick_mrpack_destination,
            commands::get_profile,
            commands::list_skins,
            commands::add_skin_from_file,
            commands::save_skin_png,
            commands::add_skin_from_player,
            commands::save_current_skin,
            commands::remove_skin,
            commands::apply_skin,
            commands::reset_skin,
            commands::set_cape,
            commands::pick_skin_file,
            commands::get_accounts,
            commands::sign_in,
            commands::cancel_sign_in,
            commands::set_active_account,
            commands::remove_account,
            commands::open_login_page,
            commands::open_external_link,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Zenith");
}
