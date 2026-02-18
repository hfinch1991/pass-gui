mod pass;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            pass::check_setup_status,
            pass::mark_setup_complete,
            pass::get_store_info,
            pass::install_dependencies,
            pass::list_gpg_keys,
            pass::generate_gpg_key,
            pass::init_password_store,
            pass::init_git_repo,
            pass::add_git_remote,
            pass::list_store,
            pass::show_password,
            pass::insert_password,
            pass::generate_password,
            pass::remove_password,
            pass::remove_batch,
            pass::move_password,
            pass::copy_password,
            pass::find_passwords,
            pass::grep_passwords,
            pass::git_push,
            pass::git_pull,
            pass::git_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
