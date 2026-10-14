//! What differs between Linux (Hyprland, Wayland) and Windows: reading the selection, the
//! focused window, the clipboard, notifications, the double-Shift trigger and the browser's
//! native messaging registration. Everything else is shared.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

pub const HOST_NAME: &str = "io.github.lautarosegura.patina";
/// Derived from the public key in extension/manifest.json.
pub const EXTENSION_ID: &str = "fedemognlmdojgmeinfhhjliccpfncdo";

pub fn host_manifest(exe: &std::path::Path) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": HOST_NAME,
        "description": "Patina: receives the page and sender of highlighted text",
        "path": exe,
        "type": "stdio",
        "allowed_origins": [format!("chrome-extension://{EXTENSION_ID}/")],
    }))
    .expect("manifest serializes")
}

/// Writes the file only when its content changed, so running this on every start is free.
pub fn write_if_changed(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}
