use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Prompt,
    Capture,
}

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
    /// The selection as HTML, when the source offered it. The UI only borrows its bold, italic,
    /// code and links, and ignores it if its characters don't match `text`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

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
    pub unknown_fields: Map<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub time: String,
    pub sender: String,
}

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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub html: String,
    pub received: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Page {
    pub kind: String,
    pub tab: String,
    pub url: String,
    pub host: String,
    pub title: String,
    pub favicon: String,
    #[serde(rename = "since")]
    pub active_since: f64,
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Window {
    pub app_id: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::{Item, Page, Selection};

    #[test]
    fn page_keeps_since_on_disk() {
        let raw = r#"{"kind":"page","tab":"7","url":"https://example.com/","host":"example.com","title":"Example","favicon":"","since":123.0,"received":124.0}"#;
        let page: Page = serde_json::from_str(raw).unwrap();
        assert_eq!(page.active_since, 123.0);
        assert_eq!(serde_json::to_string(&page).unwrap(), raw);
    }

    #[test]
    fn selection_keeps_date_order_strings_on_disk() {
        for order in ["", "dmy", "mdy", "ymd"] {
            let raw = format!(
                r#"{{"text":"hello","url":"","host":"","title":"","favicon":"","messages":[],"chat":"","date_order":"{order}","received":123.0}}"#
            );
            let selection: Selection = serde_json::from_str(&raw).unwrap();
            assert_eq!(selection.date_order, order);
            assert_eq!(serde_json::to_string(&selection).unwrap(), raw);
        }
    }

    #[test]
    fn item_keeps_unknown_fields_on_disk() {
        let raw = r#"{"id":"a1","kind":"capture","text":"hello","source":"foot","created":"2026-10-13T20:42:00","done":false,"custom":{"nested":true}}"#;
        let item: Item = serde_json::from_str(raw).unwrap();
        assert_eq!(item.unknown_fields["custom"], serde_json::json!({"nested": true}));
        assert_eq!(serde_json::to_string(&item).unwrap(), raw);
    }
}
