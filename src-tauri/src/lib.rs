mod blur;

/// Open `src`, gaussian-blur the given rectangle, and save the result to `dst`.
/// Output format is inferred from the `dst` file extension.
#[tauri::command]
fn blur_and_save(
    src: String,
    dst: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    sigma: f32,
) -> Result<(), String> {
    let img = image::open(&src).map_err(|e| e.to_string())?.to_rgba8();
    let out = blur::blur_region(&img, x, y, width, height, sigma);
    out.save(&dst).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![blur_and_save])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
