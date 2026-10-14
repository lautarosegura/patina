//! Pure capture logic, shared by both platforms. The platform layer gathers the inputs
//! (selection text, focused window, extension files, clock) and applies the result.

use std::sync::LazyLock;

use chrono::NaiveDate;
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::model::{non_empty, Context, Page, Selection, Window};

/// Selections the extension reported more than 10 minutes ago are ignored.
pub const SELECTION_MAX_AGE_S: f64 = 600.0;
/// A primary selection that wasn't captured within 10 seconds counts as gone: on Wayland it
/// stays available even after nothing looks selected on screen.
pub const PRIMARY_MAX_AGE_S: f64 = 10.0;

pub static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"https?://[^\s<>"']+"#).unwrap());
// Both WhatsApp instances are browser windows; only the title tells them apart:
// "WhatsApp - Brave" and "(19) WhatsApp Business - Brave".
static WHATSAPP_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:\(\d+\) )?(WhatsApp(?: Business)?) - ").unwrap());
static SENT_TIME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\d{1,2})[:.](\d{2})\s*([ap])?\.?\s*m?\.?").unwrap());
static SENT_DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d{1,4})[/.-](\d{1,2})[/.-](\d{1,4})").unwrap());

pub fn favicon_name(host: &str) -> String {
    let digest = Sha256::digest(host.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("{}.png", &hex[..20])
}

pub fn window_source(window: &Window) -> String {
    if let Some(m) = WHATSAPP_TITLE.captures(&window.title) {
        return m[1].to_string();
    }
    // "com.t3tools.T3Code" -> "T3Code"; simple classes stay the same.
    window.class.rsplit('.').next().unwrap_or_default().to_string()
}

/// WhatsApp's send time ("20:27, 13/10/2026" or "8:27 PM, 10/13/2026") as "YYYY-MM-DDTHH:MM".
/// The format follows the browser language; the extension sends the date order. Without it, a
/// number above 12 decides, otherwise day/month is assumed. Empty if it can't be parsed.
pub fn parse_sent(raw: &str, order: &str) -> String {
    let (Some(t), Some(d)) = (SENT_TIME.captures(raw), SENT_DATE.captures(raw)) else {
        return String::new();
    };
    let num = |s: &str| s.parse::<u32>().unwrap_or(0);
    let (mut hour, minute) = (num(&t[1]), num(&t[2]));
    if let Some(ampm) = t.get(3) {
        hour = hour % 12 + if ampm.as_str().eq_ignore_ascii_case("p") { 12 } else { 0 };
    }
    let (a, b, c) = (num(&d[1]), num(&d[2]), num(&d[3]));
    let order = match order {
        "dmy" | "mdy" | "ymd" => order,
        _ if a > 31 => "ymd",
        _ if b > 12 => "mdy",
        _ => "dmy",
    };
    let (year, month, day) = match order {
        "dmy" => (c, b, a),
        "mdy" => (c, a, b),
        _ => (a, b, c),
    };
    let year = if year > 99 { year } else { 2000 + year };
    NaiveDate::from_ymd_opt(year as i32, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, 0))
        .map(|dt| dt.format("%Y-%m-%dT%H:%M").to_string())
        .unwrap_or_default()
}

pub fn no_spaces(text: &str) -> String {
    text.split_whitespace().collect()
}

fn recent(selection: &Selection, now: f64) -> bool {
    now - selection.received <= SELECTION_MAX_AGE_S
}

/// The text as the extension built it, when it has the same characters as the captured
/// selection apart from whitespace. Chromium breaks lines around inline-flex elements such as X
/// mentions ("asking \n@bot\n:"); the extension keeps them as they look.
pub fn page_text(text: &str, selection: Option<&Selection>, now: f64) -> String {
    match selection {
        Some(s) if recent(s, now) && !s.text.trim().is_empty() && no_spaces(&s.text) == no_spaces(text) => s.text.clone(),
        _ => text.to_string(),
    }
}

/// Where the captured text came from, in this order:
/// 1. The selection the extension reported, if it is this text: the WhatsApp sender, time and
///    chat, or the page url, title and favicon.
/// 2. The browser's active tab, if it belongs to the focused window. Covers what isn't selected
///    inside a page, such as the URL in the address bar.
/// 3. If the text is a URL, that URL (with its domain's saved favicon, if any).
pub fn browser_context(
    text: &str,
    window_title: &str,
    selection: Option<&Selection>,
    page: Option<&Page>,
    now: f64,
    favicon_exists: &dyn Fn(&str) -> bool,
) -> Context {
    let captured = no_spaces(text);
    if captured.is_empty() {
        return Context::default();
    }
    let matching = selection.filter(|s| {
        let selected = no_spaces(&s.text);
        recent(s, now)
            && !selected.is_empty()
            && (captured == selected || selected.contains(&captured) || captured.contains(&selected))
    });
    let Some(selection) = matching else {
        if let Some(page) = page.filter(|p| on_page(window_title, p)) {
            return page.context();
        }
        if URL_RE.find(&captured).is_some_and(|m| m.as_str() == captured) {
            let host = url::Url::parse(&captured).ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default();
            let favicon = favicon_name(&host);
            return Context {
                url: Some(captured),
                host: non_empty(&host),
                favicon: favicon_exists(&favicon).then_some(favicon),
                ..Context::default()
            };
        }
        return Context::default();
    };
    if selection.messages.is_empty() {
        return Context {
            url: non_empty(&selection.url),
            title: non_empty(&selection.title),
            host: non_empty(&selection.host),
            favicon: non_empty(&selection.favicon),
            ..Context::default()
        };
    }
    let mut senders: Vec<&str> = Vec::new();
    for m in &selection.messages {
        if !m.sender.is_empty() && !senders.contains(&m.sender.as_str()) {
            senders.push(&m.sender);
        }
    }
    let raw_sent = &selection.messages[0].time;
    let sent = match parse_sent(raw_sent, &selection.date_order) {
        parsed if parsed.is_empty() => raw_sent.clone(),
        parsed => parsed,
    };
    // In a one-on-one chat the chat name repeats the sender.
    let chat = (!senders.contains(&selection.chat.as_str())).then(|| selection.chat.clone());
    Context {
        sender: non_empty(&senders.join(", ")),
        sent: non_empty(&sent),
        chat: chat.as_deref().and_then(non_empty),
        ..Context::default()
    }
}

/// The browser titles its window "<tab title> - <browser>".
fn on_page(window_title: &str, page: &Page) -> bool {
    !page.title.is_empty() && window_title.starts_with(&format!("{} - ", page.title))
}

/// When the current selection appeared and when the last capture consumed one. Zero means
/// unknown: without a way to watch the selection, every selection counts as fresh.
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectionClock {
    pub changed_at: f64,
    pub captured_at: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Capture {
    /// Nothing new is selected in a browser window: capture the active tab.
    Page(Page),
    Text { text: String, context: Context },
    Nothing,
}

pub struct CaptureInput<'a> {
    pub selection_text: &'a str,
    pub clock: SelectionClock,
    pub now: f64,
    pub window: &'a Window,
    pub page: Option<&'a Page>,
    pub selection: Option<&'a Selection>,
}

pub fn decide_capture(input: &CaptureInput, favicon_exists: &dyn Fn(&str) -> bool) -> Capture {
    let CaptureInput { selection_text, clock, now, window, page, selection } = *input;
    let changed = clock.changed_at;
    let mut fresh = changed == 0.0 || (changed > clock.captured_at && now - changed <= PRIMARY_MAX_AGE_S);
    let page = page.filter(|p| on_page(&window.title, p));
    if let Some(page) = page {
        // The extension reports when the page selection becomes empty (a click elsewhere).
        if selection.is_some_and(|s| s.received > changed && s.url == page.url && s.text.trim().is_empty()) {
            fresh = false;
        }
        // Selected before this tab became active: the browser doesn't update the selection
        // when the address bar is clicked.
        if changed != 0.0 && page.since > changed {
            fresh = false;
        }
    }
    if selection_text.trim().is_empty() {
        fresh = false;
    }
    match (fresh, page) {
        (false, Some(page)) => Capture::Page(page.clone()),
        (false, None) => Capture::Nothing,
        (true, _) => Capture::Text {
            text: page_text(selection_text, selection, now),
            context: browser_context(selection_text, &window.title, selection, input.page, now, favicon_exists),
        },
    }
}

/// The notification shown after a capture: (summary, body).
pub fn capture_notice(text: &str, sender: Option<&str>) -> (String, String) {
    let preview: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut body: String = preview.chars().take(120).collect();
    if preview.chars().count() > 120 {
        body.push('…');
    }
    let summary = match sender {
        Some(sender) => format!("Capturado de {sender}"),
        None => "Capturado en Patina".to_string(),
    };
    (summary, body)
}

pub struct Applied {
    /// (summary, body) to show, if any.
    pub notice: Option<(String, String)>,
    /// A text capture consumes the selection: it won't be captured again.
    pub consumed: bool,
}

pub fn apply_capture(store: &mut crate::store::Store, capture: Capture, source: &str) -> Applied {
    use crate::model::Kind;
    match capture {
        Capture::Nothing => Applied {
            notice: Some(("Patina".into(), "No hay texto seleccionado".into())),
            consumed: false,
        },
        Capture::Page(page) => {
            let added = store.add(Kind::Capture, &page.url, source, page.context());
            Applied { notice: added.then(|| ("Capturada la página".into(), page.url.clone())), consumed: false }
        }
        Capture::Text { text, context } => {
            let sender = context.sender.clone();
            let added = store.add(Kind::Capture, &text, source, context);
            Applied { notice: added.then(|| capture_notice(&text, sender.as_deref())), consumed: true }
        }
    }
}
