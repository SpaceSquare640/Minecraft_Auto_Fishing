//! Event log: the last events in memory for the UI, and - only when the player turns it on -
//! one file per day in `%LOCALAPPDATA%\<identifier>\logs\` (see [`crate::paths`]), kept for 7 days.
//! Only app events are logged: no keystrokes, screen images or audio.
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use maf_platform_win::clock;
use serde::Serialize;

const KEEP_IN_MEMORY: usize = 200;
const KEEP_FILES: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub id: u64,
    /// Local time, "HH:MM:SS".
    pub time: String,
    pub text: String,
    /// Local date, "YYYY-MM-DD": picks the log file.
    #[serde(skip)]
    date: String,
}

pub struct EventLog {
    entries: VecDeque<Entry>,
    next_id: u64,
    save: bool,
    /// Newest entry already written to a file.
    saved_through: u64,
    sink: fn(&Entry),
    prune: fn(),
}

impl EventLog {
    pub fn new(save: bool) -> Self {
        let mut log = Self {
            entries: VecDeque::new(),
            next_id: 1,
            save: false,
            saved_through: 0,
            sink: write_line,
            prune: prune_old_files,
        };
        log.set_save(save);
        log
    }

    pub fn dir() -> Option<PathBuf> {
        Some(crate::paths::local_dir()?.join("logs"))
    }

    /// Turning saving on also writes the events of this session that are still in memory.
    pub fn set_save(&mut self, save: bool) {
        self.save = save;
        if save {
            (self.prune)();
            for entry in self.entries.iter().filter(|e| e.id > self.saved_through) {
                (self.sink)(entry);
            }
            self.saved_through = self.next_id - 1;
        }
    }

    pub fn add(&mut self, text: impl Into<String>) {
        let now = clock::local_now();
        let entry = Entry {
            id: self.next_id,
            time: now.time(),
            text: text.into(),
            date: now.date(),
        };
        self.next_id += 1;
        if self.save {
            (self.sink)(&entry);
            self.saved_through = entry.id;
        }
        self.entries.push_back(entry);
        if self.entries.len() > KEEP_IN_MEMORY {
            self.entries.pop_front();
        }
    }

    /// Entries newer than `id`, oldest first.
    pub fn since(&self, id: u64) -> Vec<Entry> {
        self.entries.iter().filter(|e| e.id > id).cloned().collect()
    }
}

fn write_line(entry: &Entry) {
    let date = &entry.date;
    let Some(dir) = EventLog::dir() else { return };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{date}.log")))
    {
        let _ = writeln!(file, "{date} {} {}", entry.time, entry.text);
    }
}

/// Deletes this app's own `*.log` files older than 7 days.
fn prune_old_files() {
    let Some(dir) = EventLog::dir() else { return };
    let Ok(files) = fs::read_dir(dir) else { return };
    for file in files.flatten() {
        let path = file.path();
        let old = file
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > KEEP_FILES);
        if old && path.extension().is_some_and(|e| e == "log") {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_keeps_the_newest_entries_and_hands_out_new_ones_only() {
        let mut log = memory_log();
        for i in 0..250 {
            log.add(format!("event {i}"));
        }
        assert_eq!(log.since(0).len(), KEEP_IN_MEMORY);
        assert_eq!(log.since(0)[0].text, "event 50");
        assert_eq!(log.since(249).len(), 1);
        assert!(log.since(250).is_empty());
    }

    thread_local! {
        static WRITTEN: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn record(entry: &Entry) {
        WRITTEN.with(|w| w.borrow_mut().push(entry.text.clone()));
    }

    fn memory_log() -> EventLog {
        EventLog {
            entries: VecDeque::new(),
            next_id: 1,
            save: false,
            saved_through: 0,
            sink: record,
            prune: || {},
        }
    }

    fn written() -> Vec<String> {
        WRITTEN.with(|w| w.borrow().clone())
    }

    #[test]
    fn turning_saving_on_writes_this_session_once() {
        let mut log = memory_log();
        log.add("started");
        log.add("bite");
        assert!(written().is_empty());

        log.set_save(true);
        assert_eq!(written(), ["started", "bite"]);
        log.add("reel");
        assert_eq!(written(), ["started", "bite", "reel"]);

        // Off and on again: only what happened in between is added.
        log.set_save(false);
        log.add("cast");
        log.set_save(true);
        assert_eq!(written(), ["started", "bite", "reel", "cast"]);
    }
}
