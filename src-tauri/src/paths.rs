use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Paths {
    pub data: PathBuf,
    pub favicons: PathBuf,
    pub selection: PathBuf,
    pub page: PathBuf,
    #[cfg(target_os = "linux")]
    pub shift_stamp: PathBuf,
}

impl Paths {
    pub fn from_env() -> Self {
        let data = dirs::data_dir().unwrap_or_else(std::env::temp_dir).join("patina");
        let runtime = runtime_dir();
        Paths {
            favicons: data.join("favicons"),
            data,
            selection: runtime.join("patina-selection.json"),
            page: runtime.join("patina-page.json"),
            #[cfg(target_os = "linux")]
            shift_stamp: runtime.join("patina-shift"),
        }
    }
}

#[cfg(unix)]
fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

#[cfg(windows)]
fn runtime_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("patina");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn epoch_seconds() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}
