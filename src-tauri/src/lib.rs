// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod types;
mod commands;
pub mod services;
pub mod utils;
pub mod cli;

use commands::{data, config, autostart, shell, version, git, update, local_links, portable, hidden};
#[cfg(not(target_os = "android"))]
use commands::upload;
use tauri::Manager;
#[cfg(not(target_os = "android"))]
use tauri::{menu::{MenuBuilder, MenuItemBuilder}, tray::TrayIconBuilder};
#[cfg(not(target_os = "android"))]
use utils::cancel::PublishCancelManager;
#[cfg(not(target_os = "android"))]
use utils::lock::PublishLockManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    #[cfg(not(target_os = "android"))]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ));

    builder
        .setup(|app| {
            #[cfg(not(target_os = "android"))]
            {
                app.manage(PublishLockManager::new());
                app.manage(PublishCancelManager::new());

                let show_i = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
                let quit_i = MenuItemBuilder::with_id("quit", "退出").build(app)?;
                let menu = MenuBuilder::new(app)
                    .items(&[&show_i, &quit_i])
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .icon(app.default_window_icon().unwrap().clone())
                    .on_menu_event(move |app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            data::fetch_links,
            data::get_cache,
            data::clear_cache,
            data::fetch_version,
            config::get_upload_config,
            config::save_upload_config,
            config::get_note,
            config::save_note,
            config::get_global_config,
            config::save_global_config,
            #[cfg(not(target_os = "android"))]
            autostart::is_autostart_enabled,
            #[cfg(not(target_os = "android"))]
            autostart::enable_autostart,
            #[cfg(not(target_os = "android"))]
            autostart::disable_autostart,
            shell::resolve_hostname,
            shell::execute_command,
            shell::open_folder,
            version::fetch_version_info,
            git::get_repository_status,
            #[cfg(not(target_os = "android"))]
            upload::test_ssh_connection,
            #[cfg(not(target_os = "android"))]
            upload::publish_to_server,
            #[cfg(not(target_os = "android"))]
            upload::cancel_publish,
            update::fetch_app_version_info,
            update::get_current_version,
            update::get_current_platform,
            update::compare_versions,
            local_links::get_local_links,
            local_links::save_local_links,
            portable::export_config,
            portable::import_config,
            hidden::get_hidden_remote_links,
            hidden::save_hidden_remote_links,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
