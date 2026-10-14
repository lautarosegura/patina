mod app;
mod host;
mod logic;
mod model;
mod paths;
mod platform;
mod store;

#[cfg(test)]
mod parity_tests;

use app::{Command, USAGE};
use paths::Paths;

/// The executable has three modes, chosen before Tauri starts:
/// - `patina tap` (Linux): Hyprland calls it on every Shift release; must stay cheap.
/// - `patina chrome-extension://…`: the browser extension's native messaging host.
/// - anything else: the app, or a command forwarded to the instance already running.
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let paths = Paths::from_env();
    let rest = &args[1.min(args.len())..];
    match rest.first().map(String::as_str) {
        #[cfg(target_os = "linux")]
        Some("tap") => platform::tap(&paths),
        Some(origin) if origin.starts_with("chrome-extension://") => {
            if let Err(err) = host::run(&paths) {
                eprintln!("patina: host nativo: {err}");
            }
        }
        _ => match Command::parse(rest) {
            Some(command) => app::run(paths, command),
            None => {
                eprintln!("{USAGE}");
                std::process::exit(2);
            }
        },
    }
}
