use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Prompt,
    Capture,
}

/// Where a capture came from. On a WhatsApp message: sender, send time and chat.
/// On a web page: url, title, host and the favicon file name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
}

/// A queue or history entry. The JSON shape is the one the Python version wrote to
/// items.json and historial.json; unknown keys are kept so no data is lost on save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub kind: Kind,
    pub text: String,
    #[serde(default)]
    pub source: String,
    pub created: String,
    #[serde(default)]
    pub done: bool,
    #[serde(flatten)]
    pub context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub time: String,
    pub sender: String,
}

/// The latest selection the browser extension reported (patina-selection.json).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Selection {
    pub text: String,
    pub url: String,
    pub host: String,
    pub title: String,
    pub favicon: String,
    pub messages: Vec<Message>,
    pub chat: String,
    pub date_order: String,
    pub received: f64,
}

/// The browser's active tab (patina-page.json). `since` is when this tab and URL became
/// active; a title change keeps it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Page {
    pub kind: String,
    pub tab: String,
    pub url: String,
    pub host: String,
    pub title: String,
    pub favicon: String,
    pub since: f64,
    pub received: f64,
}

impl Page {
    pub fn context(&self) -> Context {
        Context {
            url: non_empty(&self.url),
            title: non_empty(&self.title),
            host: non_empty(&self.host),
            favicon: non_empty(&self.favicon),
            ..Context::default()
        }
    }
}

pub fn non_empty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

/// The window that had focus when the capture was requested.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Window {
    pub class: String,
    pub title: String,
}
