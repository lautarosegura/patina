use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::model::{Context, Item, Kind};

pub const ITEMS_FILE: &str = "items.json";
pub const HISTORY_FILE: &str = "historial.json";
pub const NOTES_FILE: &str = "notas.md";

pub struct Store {
    pub items: Vec<Item>,
    pub history: Vec<Item>,
    dir: Option<PathBuf>,
}

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("{}.tmp", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
    fs::write(&tmp, data)?;
    fs::rename(tmp, path)
}

fn load_list(path: &Path) -> Vec<Item> {
    let Ok(raw) = fs::read_to_string(path) else { return Vec::new() };
    if raw.trim().is_empty() {
        return Vec::new();
    }
    serde_json::from_str(&raw).unwrap_or_else(|err| {
        eprintln!("patina: {} no se pudo leer ({err}); queda como .corrupt", path.display());
        let _ = fs::rename(path, path.with_extension("json.corrupt"));
        Vec::new()
    })
}

impl Store {
    pub fn open(dir: PathBuf) -> Self {
        Store { items: load_list(&dir.join(ITEMS_FILE)), history: load_list(&dir.join(HISTORY_FILE)), dir: Some(dir) }
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        Store { items: Vec::new(), history: Vec::new(), dir: None }
    }

    fn save_list(&self, file: &str, list: &[Item]) {
        let Some(dir) = &self.dir else { return };
        let json = serde_json::to_vec_pretty(list).expect("items serialize");
        if let Err(err) = write_atomic(&dir.join(file), &json) {
            eprintln!("patina: no se pudo guardar {file}: {err}");
        }
    }

    fn save(&self) {
        self.save_list(ITEMS_FILE, &self.items);
    }

    fn save_history(&self) {
        self.save_list(HISTORY_FILE, &self.history);
    }

    pub fn add(&mut self, kind: Kind, text: &str, source: &str, context: Context) -> bool {
        let text = text.trim_matches('\n');
        if text.trim().is_empty() {
            return false;
        }
        if self.items.last().is_some_and(|last| last.text == text && !last.done) {
            return false;
        }
        self.items.push(Item {
            id: uuid::Uuid::new_v4().simple().to_string(),
            kind,
            text: text.to_string(),
            source: source.to_string(),
            created: now_iso(),
            done: false,
            context,
            archived: None,
            unknown_fields: Default::default(),
        });
        self.save();
        true
    }

    pub fn set_done(&mut self, id: &str, done: bool) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if item.done != done {
                item.done = done;
                self.save();
            }
        }
    }

    pub fn move_item(&mut self, id: &str, delta: i64) {
        let Some(i) = self.items.iter().position(|i| i.id == id) else { return };
        let j = i as i64 + delta;
        if (0..self.items.len() as i64).contains(&j) {
            self.items.swap(i, j as usize);
            self.save();
        }
    }

    pub fn archive(&mut self, ids: &[String]) {
        if ids.is_empty() {
            return;
        }
        let now = now_iso();
        let (leaving, staying): (Vec<Item>, Vec<Item>) = self.items.drain(..).partition(|i| ids.contains(&i.id));
        self.items = staying;
        self.history.extend(leaving.into_iter().map(|item| Item { archived: Some(now.clone()), ..item }));
        self.save_history();
        self.save();
    }

    pub fn delete(&mut self, id: &str) {
        if self.items.iter().any(|i| i.id == id) {
            self.archive(&[id.to_string()]);
        }
    }

    pub fn clear_done(&mut self) {
        let done: Vec<String> = self.items.iter().filter(|i| i.done).map(|i| i.id.clone()).collect();
        self.archive(&done);
    }

    pub fn restore(&mut self, id: &str) {
        let Some(i) = self.history.iter().position(|i| i.id == id) else { return };
        let item = self.history.remove(i);
        self.items.push(Item { archived: None, done: false, ..item });
        self.save();
        self.save_history();
    }

    pub fn forget(&mut self, id: &str) {
        self.history.retain(|i| i.id != id);
        self.save_history();
    }

    pub fn notes(&self) -> String {
        self.dir.as_ref().and_then(|d| fs::read_to_string(d.join(NOTES_FILE)).ok()).unwrap_or_default()
    }

    pub fn save_notes(&self, text: &str) {
        if let Some(dir) = &self.dir {
            if let Err(err) = write_atomic(&dir.join(NOTES_FILE), text.as_bytes()) {
                eprintln!("patina: no se pudieron guardar las notas: {err}");
            }
        }
    }
}
