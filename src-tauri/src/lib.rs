mod blur;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn working_path() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("parklife-{}-{}.png", std::process::id(), n))
}

/// Copy `src` into a fresh PNG working file and return its path.
#[tauri::command]
fn start_session(src: String) -> Result<String, String> {
    let img = image::open(&src).map_err(|e| e.to_string())?.to_rgba8();
    let dst = working_path();
    img.save(&dst).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().into_owned())
}

/// Blur the polygon in `working`, write the result to a new working file, return its path.
#[tauri::command]
fn apply_blur(working: String, points: Vec<(f32, f32)>, sigma: f32) -> Result<String, String> {
    let img = image::open(&working).map_err(|e| e.to_string())?.to_rgba8();
    let out = blur::blur_polygon(&img, &points, sigma);
    let dst = working_path();
    out.save(&dst).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().into_owned())
}

/// Write the current working file to `dst`, format chosen by its extension.
#[tauri::command]
fn save_copy(working: String, dst: String) -> Result<(), String> {
    image::open(&working)
        .map_err(|e| e.to_string())?
        .save(&dst)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            start_session,
            apply_blur,
            save_copy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
