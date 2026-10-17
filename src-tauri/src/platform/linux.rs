use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Wry};

use crate::logic::SelectionClock;
use crate::model::Window;
use crate::paths::{epoch_seconds, Paths};

const DOUBLE_TAP_MS: u64 = 350;

pub fn plugins(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder
}

pub fn start(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<crate::app::AppState>();
    watch_primary(state.clock.clone());
    if let Err(err) = register_native_host() {
        eprintln!("patina: no se pudo registrar el host nativo: {err}");
    }
    Ok(())
}

fn stdout_or_empty(program: &str, args: &[&str], timeout: Duration) -> String {
    let Ok(mut child) = Command::new(program).args(args).stdout(Stdio::piped()).stderr(Stdio::null()).spawn() else {
        return String::new();
    };
    // Drain stdout while waiting: a selection larger than the pipe buffer would block the child.
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        let _ = std::io::Read::read_to_string(&mut stdout, &mut out);
        out
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return reader.join().unwrap_or_default(),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return String::new();
            }
        }
    }
}

pub fn active_window() -> Window {
    let raw = stdout_or_empty("hyprctl", &["activewindow", "-j"], Duration::from_secs(2));
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
    let field = |k: &str| json.get(k).and_then(|v| v.as_str()).unwrap_or_default().to_string();
    Window { app_id: field("class"), title: field("title") }
}

/// The selected text, its HTML if the source app offers one (browsers, LibreOffice; terminals
/// don't), and the clock.
pub fn selection(clock: &Arc<Mutex<SelectionClock>>) -> (String, String, SelectionClock) {
    let text = stdout_or_empty("wl-paste", &["--primary", "--no-newline", "--type", "text"], Duration::from_secs(2));
    let html = if text.trim().is_empty() {
        String::new()
    } else {
        stdout_or_empty("wl-paste", &["--primary", "--no-newline", "--type", "text/html"], Duration::from_secs(1))
    };
    (text, html, *clock.lock().unwrap())
}

fn consume_initial_selection(reader: &mut impl BufRead, clock: &Mutex<SelectionClock>) -> bool {
    let mut line = String::new();
    if !reader.read_line(&mut line).is_ok_and(|read| read > 0) {
        return false;
    }
    let mut clock = clock.lock().unwrap();
    clock.changed_at = Some(epoch_seconds());
    clock.captured_at = clock.changed_at;
    true
}

fn watch_primary(clock: Arc<Mutex<SelectionClock>>) {
    let mut command = Command::new("wl-paste");
    command.args(["--primary", "--watch", "echo"]).stdout(Stdio::piped()).stderr(Stdio::null());
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
            Ok(())
        });
    }
    let Ok(mut child) = command.spawn() else {
        eprintln!("patina: wl-paste no está disponible; toda selección cuenta como nueva");
        return;
    };
    let stdout = child.stdout.take().unwrap();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        if consume_initial_selection(&mut reader, &clock) {
            for line in reader.lines() {
                if line.is_err() {
                    break;
                }
                clock.lock().unwrap().changed_at = Some(epoch_seconds());
            }
        }
        let _ = child.wait();
    });
}

pub fn copy(_app: &AppHandle, text: &str) {
    // wl-copy forks and keeps serving the clipboard in the background.
    let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
    else {
        return;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
}

pub fn notify(_app: &AppHandle, summary: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "Patina", "-t", "1800", "-h", "string:x-dunst-stack-tag:patina", summary, body])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn monotonic_ms() -> u64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
    ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000
}

pub fn tap(paths: &Paths) {
    let now = monotonic_ms();
    let last: u64 = std::fs::read_to_string(&paths.shift_stamp).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    if now >= last && now - last < DOUBLE_TAP_MS {
        let _ = std::fs::remove_file(&paths.shift_stamp);
        if let Ok(exe) = std::env::current_exe() {
            let err = Command::new(exe).arg("capture").exec();
            eprintln!("patina: {err}");
        }
        return;
    }
    let _ = std::fs::write(&paths.shift_stamp, now.to_string());
}

/// The path browsers should launch. Inside an AppImage the executable lives in a temporary
/// mount, so the AppImage file itself is used.
fn host_executable() -> std::io::Result<PathBuf> {
    match std::env::var_os("APPIMAGE") {
        Some(appimage) => Ok(PathBuf::from(appimage)),
        None => std::env::current_exe(),
    }
}

fn register_native_host() -> std::io::Result<()> {
    let manifest = super::host_manifest(&host_executable()?);
    let config = dirs::config_dir().unwrap_or_default();
    for browser in ["BraveSoftware/Brave-Browser", "google-chrome", "chromium"] {
        let dir: &Path = &config.join(browser);
        if dir.is_dir() {
            super::write_if_changed(&dir.join("NativeMessagingHosts").join(format!("{}.json", super::HOST_NAME)), &manifest)?;
        }
    }
    Ok(())
}
