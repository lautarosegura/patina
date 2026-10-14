use std::sync::{Arc, Mutex};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_opener::OpenerExt;

use crate::host::read_json;
use crate::logic::{apply_capture, decide_capture, window_source, CaptureInput, SelectionClock};
use crate::model::{Item, Kind, Page, Selection};
use crate::paths::{epoch_seconds, Paths};
use crate::platform;
use crate::store::Store;

pub const USAGE: &str = "Uso:
  patina              muestra u oculta la ventana
  patina capture      guarda el texto seleccionado
  patina add TEXTO    agrega un prompt a la cola
  patina --service    arranca en segundo plano sin mostrar la ventana";

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Toggle,
    Service,
    Capture,
    Add(String),
}

impl Command {
    pub fn parse(args: &[String]) -> Option<Command> {
        match args.first().map(String::as_str) {
            None | Some("toggle") => Some(Command::Toggle),
            Some("--service") => Some(Command::Service),
            Some("capture") => Some(Command::Capture),
            Some("add") if args.len() > 1 => Some(Command::Add(args[1..].join(" "))),
            _ => None,
        }
    }
}

pub struct AppState {
    pub paths: Paths,
    pub store: Mutex<Store>,
    pub clock: Arc<Mutex<SelectionClock>>,
}

const STATE_CHANGED: &str = "state-changed";

fn main_window(app: &AppHandle) -> tauri::WebviewWindow {
    app.get_webview_window("main").expect("main window exists after setup")
}

pub fn toggle(app: &AppHandle) {
    let window = main_window(app);
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show(app);
    }
}

pub fn show(app: &AppHandle) {
    let window = main_window(app);
    // Mapping the window again makes Hyprland focus it even if it was already visible.
    let _ = window.hide();
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    // The webview inside needs the keyboard too, so the prompt entry takes typing right away.
    let _ = AsRef::<tauri::Webview>::as_ref(&window).set_focus();
    let _ = app.emit("window-shown", ());
}

pub fn handle(app: &AppHandle, command: Command) {
    match command {
        Command::Service => {}
        Command::Toggle => toggle(app),
        Command::Add(text) => {
            let state = app.state::<AppState>();
            if state.store.lock().unwrap().add(Kind::Prompt, &text, "", Default::default()) {
                let _ = app.emit(STATE_CHANGED, ());
            }
        }
        Command::Capture => {
            let app = app.clone();
            std::thread::spawn(move || capture(&app));
        }
    }
}

pub fn capture(app: &AppHandle) {
    let state = app.state::<AppState>();
    let window = platform::active_window();
    let (text, clock) = platform::selection(&state.clock);
    let page: Option<Page> = read_json(&state.paths.page);
    let selection: Option<Selection> = read_json(&state.paths.selection);
    let now = epoch_seconds();
    let input = CaptureInput {
        selection_text: &text,
        clock,
        now,
        window: &window,
        page: page.as_ref(),
        selection: selection.as_ref(),
    };
    let favicons = &state.paths.favicons;
    let decision = decide_capture(&input, &|name| favicons.join(name).exists());
    let applied = apply_capture(&mut state.store.lock().unwrap(), decision, &window_source(&window));
    if applied.consumed {
        state.clock.lock().unwrap().captured_at = Some(now);
    }
    if let Some((summary, body)) = applied.notice {
        platform::notify(app, &summary, &body);
    }
    let _ = app.emit(STATE_CHANGED, ());
}

#[derive(Serialize)]
struct UiState {
    items: Vec<Item>,
    history: Vec<Item>,
    notes: String,
}

type Ui<'a> = State<'a, AppState>;

#[tauri::command]
fn get_state(state: Ui) -> UiState {
    let store = state.store.lock().unwrap();
    UiState { items: store.items.clone(), history: store.history.clone(), notes: store.notes() }
}

#[tauri::command]
fn add_prompt(state: Ui, text: String) -> bool {
    state.store.lock().unwrap().add(Kind::Prompt, &text, "", Default::default())
}

#[tauri::command]
fn set_done(state: Ui, id: String, done: bool) {
    state.store.lock().unwrap().set_done(&id, done);
}

#[tauri::command]
fn delete_item(state: Ui, id: String) {
    state.store.lock().unwrap().delete(&id);
}

#[tauri::command]
fn move_item(state: Ui, id: String, delta: i64) {
    state.store.lock().unwrap().move_item(&id, delta.signum());
}

#[tauri::command]
fn clear_done(state: Ui) {
    state.store.lock().unwrap().clear_done();
}

#[tauri::command]
fn restore_item(state: Ui, id: String) {
    state.store.lock().unwrap().restore(&id);
}

#[tauri::command]
fn forget_item(state: Ui, id: String) {
    state.store.lock().unwrap().forget(&id);
}

#[tauri::command]
fn save_notes(state: Ui, text: String) {
    state.store.lock().unwrap().save_notes(&text);
}

#[tauri::command]
fn copy_text(app: AppHandle, text: String, hide_after_copy: bool) {
    platform::copy(&app, &text);
    if hide_after_copy {
        let _ = main_window(&app).hide();
    }
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    let _ = main_window(&app).hide();
}

#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("solo se abren links http(s)".into());
    }
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn favicon_url(state: Ui, name: String) -> Option<String> {
    if !crate::logic::is_favicon_file_name(&name) {
        return None;
    }
    let png = std::fs::read(state.paths.favicons.join(&name)).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
}

pub fn run(paths: Paths, command: Command) {
    // WebKitGTK's DMA-BUF renderer kills the app with "Error 71 (Protocol error)" on Wayland with
    // NVIDIA drivers (reproduced on Hyprland + RTX 3090). The fallback renderer is plenty here.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    let store = Store::open(paths.data.clone());
    let state = AppState { paths, store: Mutex::new(store), clock: Arc::new(Mutex::new(SelectionClock::default())) };
    let builder = tauri::Builder::default()
        // Must be the first plugin: a second launch forwards its arguments here and exits.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| match Command::parse(&argv[1..]) {
            Some(command) => handle(app, command),
            None => eprintln!("{USAGE}"),
        }))
        .plugin(tauri_plugin_opener::init());
    let app = platform::plugins(builder)
        .manage(state)
        .setup(move |app| {
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Patina")
                .inner_size(560.0, 680.0)
                .decorations(false)
                .visible(false)
                .center()
                .build()?;
            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = main_window(&app_handle).hide();
                }
            });
            platform::start(app.handle())?;
            handle(app.handle(), command.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            add_prompt,
            set_done,
            delete_item,
            move_item,
            clear_done,
            restore_item,
            forget_item,
            save_notes,
            copy_text,
            hide_window,
            open_url,
            favicon_url
        ])
        .build(tauri::generate_context!())
        .expect("Patina no pudo arrancar");
    app.run(|_app, event| {
        if let tauri::RunEvent::ExitRequested { api, code: None, .. } = event {
            api.prevent_exit();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::Command;

    fn parse(args: &[&str]) -> Option<Command> {
        Command::parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn commands_match_the_python_cli() {
        assert_eq!(parse(&[]), Some(Command::Toggle));
        assert_eq!(parse(&["toggle"]), Some(Command::Toggle));
        assert_eq!(parse(&["--service"]), Some(Command::Service));
        assert_eq!(parse(&["capture"]), Some(Command::Capture));
        assert_eq!(parse(&["capture", "extra"]), Some(Command::Capture));
        assert_eq!(parse(&["add", "hola", "mundo"]), Some(Command::Add("hola mundo".into())));
        assert_eq!(parse(&["add"]), None);
        assert_eq!(parse(&["foo"]), None);
    }
}
