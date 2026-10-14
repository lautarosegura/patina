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
