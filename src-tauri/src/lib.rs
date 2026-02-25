mod commands;
mod entry;
mod git;
mod gpg;
mod import_export;
mod native_messaging;
mod platform;
mod store;
mod totp;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Existing commands (Phase 1)
            commands::check_setup_status,
            commands::mark_setup_complete,
            commands::get_store_info,
            commands::install_dependencies,
            commands::list_gpg_keys,
            commands::generate_gpg_key,
            commands::init_password_store,
            commands::init_git_repo,
            commands::add_git_remote,
            commands::list_store,
            commands::show_password,
            commands::insert_password,
            commands::generate_password,
            commands::remove_password,
            commands::remove_batch,
            commands::move_password,
            commands::copy_password,
            commands::find_passwords,
            commands::grep_passwords,
            commands::git_push,
            commands::git_pull,
            commands::git_log,
            // Phase 2: Structured entries + TOTP
            commands::get_entry_fields,
            commands::save_entry_fields,
            commands::get_totp_code,
            // Phase 3: Import/Export
            commands::import_csv,
            commands::export_csv,
            // Phase 4: Browser extension search
            commands::search_by_url,
            commands::install_browser_extension,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_native_messaging() {
    native_messaging::handle_native_messaging();
}
