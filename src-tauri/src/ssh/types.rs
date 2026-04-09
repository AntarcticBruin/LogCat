use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Auth {
    Password { password: String },
    Key {
        key_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Clone, Deserialize)]
pub struct ConnectOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: Auth,
    pub keepalive_ms: Option<u32>,
}

#[derive(Serialize)]
pub struct ConnectResult {
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub is_text: bool,
    pub size: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct TailEvent {
    pub token: String,
    pub session_id: String,
    pub path: String,
    pub chunk: String,
}

#[derive(Serialize, Clone)]
pub struct TerminalEvent {
    pub token: String,
    pub session_id: String,
    pub chunk: String,
}

#[derive(Serialize, Clone)]
pub struct TransferProgressEvent {
    pub session_id: String,
    pub file_name: String,
    pub transferred: u64,
    pub total: u64,
}
