mod commands;
mod errors;
mod models;
mod services;
mod state;

use commands::{check_server_health, get_app_config, list_plugins, update_app_config};
use services::config::ConfigStore;
use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let store = ConfigStore::from_app_data_dir(app_data_dir);
            let state = AppState::load(store)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            update_app_config,
            check_server_health,
            list_plugins,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Realestate Management Apps");
}
