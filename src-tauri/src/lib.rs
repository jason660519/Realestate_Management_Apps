mod commands;
mod errors;
mod models;
mod services;
mod state;

use commands::{
    check_server_health, get_app_config, get_storage_diagnostics, list_plugins,
    list_property_summaries, update_app_config,
};
use services::{config::ConfigStore, local_db};
use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let store = ConfigStore::from_app_data_dir(app_data_dir.clone());
            let pool = tauri::async_runtime::block_on(local_db::open(app_data_dir))?;
            let state = AppState::load(store, pool)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            update_app_config,
            get_storage_diagnostics,
            check_server_health,
            list_plugins,
            list_property_summaries,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Realestate Management Apps");
}
