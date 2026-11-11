mod ai;
mod alerts;
mod app_ai;
mod appdata;
mod apps;
mod cleaner;
mod disk;
mod dupes;
mod file_ai;
mod file_rules;
mod fsutil;
mod leftovers;
mod orphans;
mod process_ai;
mod processes;
mod settings;
mod suspicious;
mod tray;
mod uninstall;

use std::sync::Mutex;

use sysinfo::System;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        // Giữ một System duy nhất để sysinfo tính được % CPU giữa hai lần làm mới.
        .manage(Mutex::new(System::new()))
        .setup(|app| {
            app.manage(Mutex::new(settings::load(app.handle())));
            alerts::spawn(app.handle().clone());
            tray::setup(app)?;
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            processes::snapshot,
            processes::kill_process,
            process_ai::analyze_process,
            cleaner::scan_junk,
            cleaner::clean_junk,
            appdata::appdata_folders,
            appdata::explain_folder,
            apps::list_apps,
            app_ai::explain_app,
            uninstall::uninstall_app,
            leftovers::find_leftovers,
            leftovers::delete_leftover,
            settings::get_settings,
            settings::save_settings,
            alerts::test_notification,
            disk::list_drives,
            disk::scan_path,
            file_rules::delete_file,
            dupes::find_duplicates,
            file_ai::explain_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
