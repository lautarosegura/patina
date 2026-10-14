#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

pub const HOST_NAME: &str = "io.github.lautarosegura.patina";
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

pub fn write_if_changed(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    #[test]
    fn extension_id_matches_manifest_key() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../extension/manifest.json");
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).expect("extension manifest")).expect("valid manifest JSON");
        let key = base64::engine::general_purpose::STANDARD
            .decode(manifest["key"].as_str().expect("extension public key"))
            .expect("base64 DER public key");
        let id: String = Sha256::digest(key)[..16]
            .iter()
            .flat_map(|byte| [char::from(b'a' + (byte >> 4)), char::from(b'a' + (byte & 0x0f))])
            .collect();
        assert_eq!(id, super::EXTENSION_ID);
    }
}
