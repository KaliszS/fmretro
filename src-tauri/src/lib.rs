mod patcher;

use patcher::{DebugInfo, PatchManager, PatchStatus};
use tauri::WindowEvent;

#[tauri::command]
fn detect_fm(state: tauri::State<PatchManager>) -> Option<u32> {
    state.detect()
}

#[tauri::command]
fn get_status(state: tauri::State<PatchManager>) -> PatchStatus {
    state.status()
}

#[tauri::command]
fn get_debug(state: tauri::State<PatchManager>) -> DebugInfo {
    state.debug()
}

#[tauri::command]
fn apply_patch(
    shift_back: i32,
    min_year: i32,
    state: tauri::State<PatchManager>,
) -> Result<String, String> {
    state.apply(shift_back, min_year)
}

#[tauri::command]
fn restore_patch(state: tauri::State<PatchManager>) -> Result<String, String> {
    state.restore()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let manager = PatchManager::new();
    let manager_for_close = manager.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(manager)
        .invoke_handler(tauri::generate_handler![
            detect_fm,
            get_status,
            get_debug,
            apply_patch,
            restore_patch
        ])
        .on_window_event(move |_window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let _ = manager_for_close.restore();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
