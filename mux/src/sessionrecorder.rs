use crate::pane::PaneId;
use chrono::Local;
use parking_lot::Mutex;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SessionMetadata {
    pub pane_id: PaneId,
    pub tab_id: Option<usize>,
    pub window_id: Option<usize>,
    pub workspace: Option<String>,
    pub title_at_start: Option<String>,
    pub cwd_at_start: Option<String>,
}

#[derive(Debug)]
struct ActiveSession {
    session_id: String,
    dir: PathBuf,
    seq: u64,
    meta: SessionMetadata,
    started_at: String,
}

pub struct SessionRecorder {
    base_dir: PathBuf,
    sessions: Mutex<HashMap<PaneId, ActiveSession>>,
}

impl SessionRecorder {
    pub fn new() -> Self {
        let base_dir = dirs_next::download_dir()
            .unwrap_or_else(|| config::HOME_DIR.clone())
            .join("wezterm")
            .join("sessions");
        let _ = fs::create_dir_all(&base_dir);
        Self {
            base_dir,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn start_session(&self, meta: SessionMetadata) {
        let started_at = now_rfc3339();
        let session_id = format!(
            "{}-pane-{}",
            Local::now().format("%Y%m%d-%H%M%S"),
            meta.pane_id
        );
        let dir = self.base_dir.join(&session_id);
        if fs::create_dir_all(&dir).is_err() {
            return;
        }

        let meta_json = json!({
            "session_id": session_id,
            "pane_id": meta.pane_id,
            "tab_id": meta.tab_id,
            "window_id": meta.window_id,
            "workspace": meta.workspace,
            "title_at_start": meta.title_at_start,
            "cwd_at_start": meta.cwd_at_start,
            "started_at": started_at,
        });
        let _ = fs::write(
            dir.join("meta.json"),
            serde_json::to_string_pretty(&meta_json).unwrap_or_else(|_| "{}".to_string()),
        );

        let mut md = String::new();
        md.push_str(&format!("# Session {}\n\n", session_id));
        md.push_str(&format!("- Started: {}\n", started_at));
        md.push_str(&format!("- Pane ID: {}\n", meta.pane_id));
        if let Some(tab_id) = meta.tab_id {
            md.push_str(&format!("- Tab ID: {}\n", tab_id));
        }
        if let Some(window_id) = meta.window_id {
            md.push_str(&format!("- Window ID: {}\n", window_id));
        }
        if let Some(workspace) = &meta.workspace {
            md.push_str(&format!("- Workspace: {}\n", workspace));
        }
        if let Some(cwd) = &meta.cwd_at_start {
            md.push_str(&format!("- Start CWD: {}\n", cwd));
        }
        if let Some(title) = &meta.title_at_start {
            md.push_str(&format!("- Title: {}\n", title));
        }
        md.push('\n');
        let _ = fs::write(dir.join("session.md"), md);

        let pane_id = meta.pane_id;
        self.sessions.lock().insert(
            pane_id,
            ActiveSession {
                session_id,
                dir,
                seq: 0,
                meta,
                started_at,
            },
        );

        self.append_event(pane_id, "session_started", json!({}));
    }

    pub fn record_input_bytes(&self, pane_id: PaneId, bytes: &[u8]) {
        self.append_text_event(pane_id, "input_raw", bytes);
        if let Some(text) = normalize_input_bytes(bytes) {
            self.append_event(pane_id, "input_text", json!({ "text": text }));
            self.append_markdown_block(pane_id, "input_text", &text);
        }
    }

    pub fn record_output_bytes(&self, pane_id: PaneId, bytes: &[u8]) {
        self.append_text_event(pane_id, "output_raw", bytes);
        if let Some(text) = normalize_output_bytes(bytes) {
            self.append_event(pane_id, "output_text", json!({ "text": text }));
            self.append_markdown_block(pane_id, "output_text", &text);
        }
    }

    pub fn record_text_event(&self, pane_id: PaneId, event_type: &str, text: &str) {
        if text.is_empty() {
            return;
        }
        self.append_event(pane_id, event_type, json!({ "text": text }));
        self.append_markdown_block(pane_id, event_type, text);
    }

    pub fn end_session(&self, pane_id: PaneId, reason: &str) {
        self.append_event(pane_id, "session_ended", json!({ "reason": reason }));

        let Some(session) = self.sessions.lock().remove(&pane_id) else {
            return;
        };

        let meta_json = json!({
            "session_id": session.session_id,
            "pane_id": session.meta.pane_id,
            "tab_id": session.meta.tab_id,
            "window_id": session.meta.window_id,
            "workspace": session.meta.workspace,
            "title_at_start": session.meta.title_at_start,
            "cwd_at_start": session.meta.cwd_at_start,
            "started_at": session.started_at,
            "ended_at": now_rfc3339(),
        });
        let _ = fs::write(
            session.dir.join("meta.json"),
            serde_json::to_string_pretty(&meta_json).unwrap_or_else(|_| "{}".to_string()),
        );
    }

    fn append_text_event(&self, pane_id: PaneId, event_type: &str, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes).to_string();
        if text.is_empty() {
            return;
        }
        self.append_event(pane_id, event_type, json!({ "text": text }));
    }

    fn append_event(&self, pane_id: PaneId, event_type: &str, data: serde_json::Value) {
        let mut sessions = self.sessions.lock();
        let Some(session) = sessions.get_mut(&pane_id) else {
            return;
        };
        session.seq += 1;

        let line = json!({
            "ts": now_rfc3339(),
            "seq": session.seq,
            "session_id": session.session_id,
            "pane_id": session.meta.pane_id,
            "tab_id": session.meta.tab_id,
            "window_id": session.meta.window_id,
            "workspace": session.meta.workspace,
            "type": event_type,
            "data": data,
        });

        append_line(
            &session.dir.join("session.jsonl"),
            &serde_json::to_string(&line).unwrap_or_else(|_| "{}".to_string()),
        );
    }

    fn append_markdown_block(&self, pane_id: PaneId, event_type: &str, text: &str) {
        let sessions = self.sessions.lock();
        let Some(session) = sessions.get(&pane_id) else {
            return;
        };

        let title = match event_type {
            "input_text" | "input_original" | "input_effective" => "Input",
            "output_text" => "Output",
            "rag_context" => "RAG Context",
            other => other,
        };

        let block = format!(
            "## {} {}\n```text\n{}\n```\n",
            now_rfc3339(),
            title,
            text.trim_end_matches('\n')
        );
        append_line(&session.dir.join("session.md"), &block);
    }
}

fn append_line(path: &Path, line: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}

fn now_rfc3339() -> String {
    Local::now().to_rfc3339()
}

fn normalize_input_bytes(bytes: &[u8]) -> Option<String> {
    let stripped = strip_terminal_sequences(bytes);
    let mut out = String::new();
    for ch in stripped.chars() {
        match ch {
            '\u{8}' | '\u{7f}' => {
                out.pop();
            }
            '\r' | '\n' => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            }
            '\t' => out.push('\t'),
            c if !c.is_control() => out.push(c),
            _ => {}
        }
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn normalize_output_bytes(bytes: &[u8]) -> Option<String> {
    let stripped = strip_terminal_sequences(bytes);
    let text = normalize_rendered_text(&stripped);
    if looks_like_noise(&text) {
        None
    } else {
        Some(text)
    }
}

fn normalize_rendered_text(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    continue;
                }
                out.push('\n');
            }
            '\n' => out.push('\n'),
            '\t' => out.push('\t'),
            c if !c.is_control() => out.push(c),
            _ => {}
        }
    }

    let mut normalized_lines = vec![];
    for line in out.lines() {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            normalized_lines.push(trimmed);
        }
    }
    normalized_lines.join("\n").trim().to_string()
}

fn looks_like_noise(s: &str) -> bool {
    if s.trim().is_empty() {
        return true;
    }

    let meaningful = s
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || is_common_punctuation(*c))
        .count();
    meaningful == 0
}

fn is_common_punctuation(c: char) -> bool {
    matches!(
        c,
        '.'
            | ','
            | ':'
            | ';'
            | '!'
            | '?'
            | '/'
            | '\\'
            | '-'
            | '_'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '|'
            | '\''
            | '"'
            | '+'
            | '='
            | '`'
            | '~'
    )
}

fn strip_terminal_sequences(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0x1b => {
                i += 1;
                if i >= bytes.len() {
                    break;
                }
                match bytes[i] {
                    b'[' => {
                        i += 1;
                        while i < bytes.len() {
                            let b = bytes[i];
                            i += 1;
                            if (0x40..=0x7e).contains(&b) {
                                break;
                            }
                        }
                    }
                    b']' => {
                        i += 1;
                        while i < bytes.len() {
                            if bytes[i] == 0x07 {
                                i += 1;
                                break;
                            }
                            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                    }
                    b'P' | b'X' | b'^' | b'_' => {
                        i += 1;
                        while i < bytes.len() {
                            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}
