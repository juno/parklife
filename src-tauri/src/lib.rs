mod blur;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn working_path() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("parklife-{}-{}.png", std::process::id(), n))
}

fn is_heic(path: &str) -> bool {
    PathBuf::from(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("heic") || e.eq_ignore_ascii_case("heif"))
}

/// `image` can't decode HEIC/HEIF. On macOS, transcode to a temp PNG via the
/// built-in `sips` and return that path; otherwise return `src` unchanged.
fn decodable_path(src: &str) -> Result<PathBuf, String> {
    if !is_heic(src) {
        return Ok(PathBuf::from(src));
    }
    // Canonicalize so the path can't start with `-` and be read as a sips flag.
    let src = std::fs::canonicalize(src).map_err(|e| e.to_string())?;
    let dst = working_path();
    let ok = std::process::Command::new("sips")
        .args(["-s", "format", "png"])
        .arg(&src)
        .arg("--out")
        .arg(&dst)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok {
        Ok(dst)
    } else {
        Err(format!("sips could not convert {}", src.display()))
    }
}

/// Copy `src` into a fresh PNG working file and return its path.
#[tauri::command]
fn start_session(src: String) -> Result<String, String> {
    let img = image::open(decodable_path(&src)?)
        .map_err(|e| e.to_string())?
        .to_rgba8();
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

/// Force a `.jpg` extension: every saved copy is JPEG.
fn jpeg_path(dst: &str) -> PathBuf {
    let mut p = PathBuf::from(dst);
    p.set_extension("jpg");
    p
}

/// Write the current working file to `dst` as JPEG (alpha dropped).
#[tauri::command]
fn save_copy(working: String, dst: String) -> Result<(), String> {
    let img = image::open(&working).map_err(|e| e.to_string())?.to_rgb8();
    img.save(jpeg_path(&dst)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{is_heic, jpeg_path};

    #[test]
    fn forces_jpg_extension() {
        assert_eq!(jpeg_path("/a/photo.png").to_str().unwrap(), "/a/photo.jpg");
        assert_eq!(jpeg_path("/a/photo").to_str().unwrap(), "/a/photo.jpg");
        assert_eq!(jpeg_path("/a/my.photo.png").to_str().unwrap(), "/a/my.photo.jpg");
    }

    #[test]
    fn detects_heic_extension() {
        assert!(is_heic("/a/photo.heic"));
        assert!(is_heic("/a/photo.HEIC"));
        assert!(is_heic("/a/photo.heif"));
        assert!(!is_heic("/a/photo.png"));
        assert!(!is_heic("/a/photo.jpg"));
        assert!(!is_heic("/a/photo"));
    }
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
