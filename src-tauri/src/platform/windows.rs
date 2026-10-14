use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_C,
    VK_CONTROL, VK_LSHIFT, VK_RSHIFT, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, GetWindowTextW, GetWindowThreadProcessId, SetWindowsHookExW,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::logic::SelectionClock;
use crate::model::Window;
use crate::paths::epoch_seconds;

const DOUBLE_TAP: Duration = Duration::from_millis(350);
const AUTOSTART_MARKER: &str = "autostart-configured";

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn plugins(builder: tauri::Builder<Wry>) -> tauri::Builder<Wry> {
    builder
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        crate::app::toggle(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--service"])))
}

pub fn start(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let _ = APP.set(app.clone());
    if let Err(err) = register_native_host() {
        eprintln!("patina: no se pudo registrar el host nativo: {err}");
    }
    let shortcuts = app.global_shortcut();
    if shortcuts.register(Shortcut::new(Some(Modifiers::SUPER), Code::KeyC)).is_err() {
        shortcuts.register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyC))?;
    }
    tray(app)?;
    enable_autostart_once(app);
    std::thread::spawn(keyboard_hook);
    Ok(())
}

fn tray(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Mostrar u ocultar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    TrayIconBuilder::with_id("patina")
        .icon(app.default_window_icon().cloned().expect("bundle icon"))
        .tooltip("Patina")
        .menu(&Menu::with_items(app, &[&toggle, &quit])?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => crate::app::toggle(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                crate::app::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn enable_autostart_once(app: &AppHandle) {
    let marker = app.state::<crate::app::AppState>().paths.data.join(AUTOSTART_MARKER);
    if marker.exists() {
        return;
    }
    if app.autolaunch().enable().is_ok() {
        let _ = std::fs::create_dir_all(marker.parent().unwrap());
        let _ = std::fs::write(marker, "");
    }
}

#[derive(Default)]
struct Taps {
    shift_down: bool,
    only_shift_since_press: bool,
    last: Option<Instant>,
}

impl Taps {
    fn press_shift(&mut self) {
        if !self.shift_down {
            self.shift_down = true;
            self.only_shift_since_press = true;
        }
    }

    fn release_shift(&mut self) -> bool {
        self.shift_down = false;
        if !self.only_shift_since_press {
            return false;
        }
        match self.last {
            Some(last) if last.elapsed() < DOUBLE_TAP => {
                self.last = None;
                true
            }
            _ => {
                self.last = Some(Instant::now());
                false
            }
        }
    }

    fn press_other(&mut self) {
        self.only_shift_since_press = false;
        self.last = None;
    }
}

static TAPS: Mutex<Taps> = Mutex::new(Taps { shift_down: false, only_shift_since_press: false, last: None });

fn is_shift(vk: u32) -> bool {
    [VK_SHIFT, VK_LSHIFT, VK_RSHIFT].iter().any(|k| k.0 as u32 == vk)
}

unsafe extern "system" fn on_key(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let key = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if key.flags.0 & LLKHF_INJECTED.0 == 0 {
            let message = wparam.0 as u32;
            let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            let up = message == WM_KEYUP || message == WM_SYSKEYUP;
            let mut taps = TAPS.lock().unwrap();
            if is_shift(key.vkCode) {
                if down {
                    taps.press_shift();
                } else if up && taps.release_shift() {
                    if let Some(app) = APP.get() {
                        let app = app.clone();
                        std::thread::spawn(move || crate::app::capture(&app));
                    }
                }
            } else if down {
                taps.press_other();
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn keyboard_hook() {
    unsafe {
        if SetWindowsHookExW(WH_KEYBOARD_LL, Some(on_key), None, 0).is_err() {
            eprintln!("patina: no se pudo instalar el hook de teclado; el doble Shift no va a funcionar");
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {}
    }
}

pub fn active_window() -> Window {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title).max(0) as usize;
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        Window { app_id: process_name(pid).unwrap_or_default(), title: String::from_utf16_lossy(&title[..len]) }
    }
}

fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = CloseHandle(process);
        ok.ok()?;
        let path = PathBuf::from(String::from_utf16_lossy(&buf[..size as usize]));
        path.file_stem().map(|s| s.to_string_lossy().into_owned())
    }
}

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn copy_selection() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let saved_text = clipboard.get_text().ok();
    let saved_image = if saved_text.is_none() { clipboard.get_image().ok() } else { None };
    // Let the target app see the Shift release before the synthetic Ctrl+C.
    std::thread::sleep(Duration::from_millis(40));
    let before = unsafe { GetClipboardSequenceNumber() };
    let inputs = [key(VK_CONTROL, false), key(VK_C, false), key(VK_C, true), key(VK_CONTROL, true)];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    let deadline = Instant::now() + Duration::from_millis(400);
    while unsafe { GetClipboardSequenceNumber() } == before {
        if Instant::now() > deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(30));
    let copied = clipboard.get_text().ok();
    let _ = match (saved_text, saved_image) {
        (Some(text), _) => clipboard.set_text(text),
        (None, Some(image)) => clipboard.set_image(image),
        (None, None) => clipboard.clear(),
    };
    copied
}

static FIRST_COPIED_AT: Mutex<Option<(String, f64)>> = Mutex::new(None);

pub fn selection(clock: &Arc<Mutex<SelectionClock>>) -> (String, SelectionClock) {
    let Some(text) = copy_selection().filter(|text| !text.trim().is_empty()) else {
        return (String::new(), SelectionClock::default());
    };
    let mut first_copied = FIRST_COPIED_AT.lock().unwrap();
    let changed_at = match &*first_copied {
        Some((seen, at)) if *seen == text => *at,
        _ => {
            let now = epoch_seconds();
            *first_copied = Some((text.clone(), now));
            now
        }
    };
    (text, SelectionClock { changed_at: Some(changed_at), captured_at: clock.lock().unwrap().captured_at })
}

pub fn copy(_app: &AppHandle, text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text.to_string());
    }
}

pub fn notify(app: &AppHandle, summary: &str, body: &str) {
    let _ = app.notification().builder().title(summary).body(body).show();
}

fn register_native_host() -> std::io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    let dir = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir).join("Patina");
    let manifest_path = dir.join(format!("{}.json", super::HOST_NAME));
    super::write_if_changed(&manifest_path, &super::host_manifest(&std::env::current_exe()?))?;
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    for browser in ["Google\\Chrome", "BraveSoftware\\Brave-Browser", "Chromium", "Microsoft\\Edge"] {
        let (key, _) = hkcu.create_subkey(format!("Software\\{browser}\\NativeMessagingHosts\\{}", super::HOST_NAME))?;
        key.set_value("", &manifest_path.to_string_lossy().to_string())?;
    }
    Ok(())
}
