use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use base64::Engine;
use serde_json::{json, Value};

use crate::logic::favicon_name;
use crate::model::{Message, Page, Selection};
use crate::paths::Paths;
use crate::store::write_atomic;

const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const MAX_FAVICON_BYTES: usize = 256_000;

pub enum Record {
    Selection(Selection),
    Page(Page),
}

fn text(data: &Value, key: &str) -> String {
    match data.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

pub fn favicon_png(encoded: &str) -> Option<Vec<u8>> {
    let png = base64::engine::general_purpose::STANDARD.decode(encoded).ok()?;
    (png.starts_with(PNG_MAGIC) && png.len() <= MAX_FAVICON_BYTES).then_some(png)
}

pub fn parse_message(
    raw: &[u8],
    previous_page: Option<&Page>,
    now: f64,
    save_favicon: &mut dyn FnMut(&str, &[u8]) -> Option<String>,
) -> Option<Record> {
    let data: Value = serde_json::from_slice(raw).ok().filter(Value::is_object)?;
    let url = text(&data, "url");
    let host = url::Url::parse(&url).ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default();
    let favicon = match data.get("favicon") {
        Some(Value::String(encoded)) if !host.is_empty() => {
            favicon_png(encoded).and_then(|png| save_favicon(&host, &png)).unwrap_or_default()
        }
        _ => String::new(),
    };
    if data.get("kind").and_then(Value::as_str) == Some("page") {
        let tab = text(&data, "tab");
        let active_since = match previous_page {
            Some(p) if p.tab == tab && p.url == url => p.active_since,
            _ => now,
        };
        return Some(Record::Page(Page {
            kind: "page".into(),
            tab,
            url,
            host,
            title: text(&data, "title"),
            favicon,
            active_since,
            received: now,
        }));
    }
    data.get("text")?;
    let messages = match data.get("messages") {
        None => Vec::new(),
        Some(Value::Array(list)) => list
            .iter()
            .map(|m| (m.get("time").is_some() && m.get("sender").is_some()).then(|| Message { time: text(m, "time"), sender: text(m, "sender") }))
            .collect::<Option<Vec<_>>>()?,
        Some(_) => return None,
    };
    let order = text(&data, "dateOrder");
    Some(Record::Selection(Selection {
        text: text(&data, "text"),
        url,
        host,
        title: text(&data, "title"),
        favicon,
        messages,
        chat: text(&data, "chat"),
        date_order: if matches!(order.as_str(), "dmy" | "mdy" | "ymd") { order } else { String::new() },
        received: now,
    }))
}

pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub fn write_private(path: &Path, value: &impl serde::Serialize) -> io::Result<()> {
    let data = serde_json::to_vec(value).expect("record serializes");
    write_atomic(path, &data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn read_frame(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    if reader.read_exact(&mut header).is_err() {
        return Ok(None);
    }
    let mut raw = vec![0u8; u32::from_ne_bytes(header) as usize];
    reader.read_exact(&mut raw)?;
    Ok(Some(raw))
}

fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    writer.write_all(&(payload.len() as u32).to_ne_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

pub fn run(paths: &Paths) -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut save_favicon = |host: &str, png: &[u8]| {
        let name = favicon_name(host);
        write_atomic(&paths.favicons.join(&name), png).ok().map(|_| name)
    };
    while let Some(raw) = read_frame(&mut stdin)? {
        let previous: Option<Page> = read_json(&paths.page);
        let now = crate::paths::epoch_seconds();
        let record = parse_message(&raw, previous.as_ref(), now, &mut save_favicon);
        let ok = match &record {
            Some(Record::Page(page)) => write_private(&paths.page, page).is_ok(),
            Some(Record::Selection(selection)) => write_private(&paths.selection, selection).is_ok(),
            None => false,
        };
        let reply = serde_json::to_vec(&json!({ "ok": ok })).expect("reply serializes");
        write_frame(&mut stdout, &reply)?;
    }
    Ok(())
}
