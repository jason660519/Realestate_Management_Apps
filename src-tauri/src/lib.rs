mod commands;
mod errors;
mod models;
mod state;

use commands::{check_server_health, get_app_config, list_plugins, update_app_config};
use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            update_app_config,
            check_server_health,
            list_plugins,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Realestate Management Apps");
}
