// Prevents additional console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod types;
mod commands;
pub mod services;
pub mod utils;

use commands::{data, config, autostart, shell, version, upload, update};
use tauri::{Manager, menu::{MenuBuilder, MenuItemBuilder}, tray::TrayIconBuilder};
use utils::lock::PublishLockManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 初始化发布锁管理器
            app.manage(PublishLockManager::new());

            // 创建系统托盘菜单
            let show_i = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
            let quit_i = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_i, &quit_i])
                .build()?;

            // 创建系统托盘
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
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            data::fetch_links,
            data::get_cache,
            data::fetch_version,
            config::get_upload_config,
            config::save_upload_config,
            config::get_note,
            config::save_note,
            autostart::is_autostart_enabled,
            autostart::enable_autostart,
            autostart::disable_autostart,
            shell::resolve_hostname,
            shell::execute_command,
            version::fetch_version_info,
            upload::test_ssh_connection,
            upload::publish_to_server,
            update::fetch_app_version_info,
            update::get_current_version,
            update::get_current_platform,
            update::compare_versions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
