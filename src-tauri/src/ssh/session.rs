use super::error::{AppError, AppResult};
use super::state::{AppState, SshConn};
use super::types::{Auth, ConnectOptions, ConnectResult};
use russh::client::{Config, Handle, Handler};
use russh::keys::key::PrivateKeyWithHashAlg;
use russh::keys::load_secret_key;
use russh::ChannelMsg;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

pub struct Client {}

impl Handler for Client {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn establish_session(opts: &ConnectOptions) -> AppResult<Handle<Client>> {
    let config = Arc::new(Config {
        keepalive_interval: opts.keepalive_ms.map(|ms| std::time::Duration::from_millis(ms as u64)),
        ..Default::default()
    });

    let addr = format!("{}:{}", opts.host, opts.port);
    let mut handle = russh::client::connect(config, addr, Client {}).await?;

    let auth_res = match &opts.auth {
        Auth::Password { password } => {
            handle.authenticate_password(&opts.username, password).await?
        }
        Auth::Key {
            key_path,
            passphrase,
        } => {
            let key = load_secret_key(key_path, passphrase.as_deref())
                .map_err(|e| AppError::Message(e.to_string()))?;
            handle
                .authenticate_publickey(
                    &opts.username,
                    PrivateKeyWithHashAlg::new(Arc::new(key), None),
                )
                .await?
        }
    };

    if !matches!(auth_res, russh::client::AuthResult::Success) {
        return Err(AppError::AuthenticationFailed);
    }

    Ok(handle)
}

pub async fn establish_for_state(opts: ConnectOptions) -> AppResult<Arc<SshConn>> {
    let handle = establish_session(&opts).await?;

    Ok(Arc::new(SshConn {
        handle: AsyncMutex::new(handle),
        opts,
        watchers: Mutex::new(HashMap::new()),
    }))
}

pub fn get_conn(state: &AppState, session_id: &str) -> AppResult<Arc<SshConn>> {
    state
        .get_session(session_id)
        .ok_or(AppError::InvalidSession)
}

pub fn connect_result_with_conn(conn: Arc<SshConn>, state: &AppState) -> ConnectResult {
    let id = Uuid::new_v4().to_string();
    state.insert_session(id.clone(), conn);
    ConnectResult { session_id: id }
}
