use super::types::ConnectOptions;
use russh::client::Handle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Default)]
pub struct AppState {
    sessions: StdMutex<HashMap<String, Arc<SshConn>>>,
}

impl AppState {
    pub fn insert_session(&self, id: String, conn: Arc<SshConn>) {
        self.sessions.lock().unwrap().insert(id, conn);
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<SshConn>> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    pub fn remove_session(&self, id: &str) -> Option<Arc<SshConn>> {
        self.sessions.lock().unwrap().remove(id)
    }
}

pub struct SshConn {
    pub handle: AsyncMutex<Handle<super::session::Client>>,
    pub opts: ConnectOptions,
    pub watchers: StdMutex<HashMap<String, Arc<Watcher>>>,
}

pub struct Watcher {
    pub stop: std::sync::atomic::AtomicBool,
}
