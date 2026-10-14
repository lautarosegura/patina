// Without a console window on Windows. The native messaging host still gets the stdin/stdout
// pipes the browser passes in.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    patina_lib::main();
}
