#[cfg(not(target_os = "android"))]
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        let _ = app;
        return Ok(false);
    }

    #[cfg(not(target_os = "android"))]
    app.autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = app;
        return Err("Android 不支持开机自启动".to_string());
    }

    #[cfg(not(target_os = "android"))]
    app.autolaunch()
        .enable()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = app;
        return Ok(());
    }

    #[cfg(not(target_os = "android"))]
    app.autolaunch()
        .disable()
        .map_err(|e| e.to_string())
}
