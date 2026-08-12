use std::io::Write;
use crate::ipc::protocol::{AiLayoutCommand, IpcResponse};
use crate::state::CompositorState;
use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub const DEFAULT_IPC_PATH: &str = "/run/ermete/compositor.sock";
pub const FALLBACK_IPC_PATH: &str = "/tmp/ermete-compositor.sock";

pub struct IpcServer {
    socket_path: PathBuf,
    state: Arc<Mutex<CompositorState>>,
}

impl IpcServer {
    pub fn new(state: Arc<Mutex<CompositorState>>) -> Self {
        let socket_path = if Path::new("/run/ermete").exists() {
            PathBuf::from(DEFAULT_IPC_PATH)
        } else {
            PathBuf::from(FALLBACK_IPC_PATH)
        };

        Self { socket_path, state }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub async fn run(&self) -> Result<()> {
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
            tracing::warn!("No previous socket to remove at {:?}: {:?}", self.socket_path, e);
        }
        }

        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create IPC socket parent dir {:?}", parent))?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("Failed to bind UNIX socket at {:?}", self.socket_path))?;

        // Set socket permissions (rw-------) for IPC security
        if let Ok(metadata) = std::fs::metadata(&self.socket_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            if let Err(e) = std::fs::set_permissions(&self.socket_path, perms) {
            tracing::error!("Failed to set permissions on socket {:?}: {:?}", self.socket_path, e);
        }
        }

        info!(
            "AI-Driven IPC Server listening on UNIX socket: {:?}",
            self.socket_path
        );

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, state).await {
                            warn!("Error handling IPC client stream: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting IPC socket connection: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn handle_client(
        stream: UnixStream,
        state: Arc<Mutex<CompositorState>>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<AiLayoutCommand>(line) {
                Ok(cmd) => {
                    let mut lock = state.lock().await;
                    lock.process_command(cmd).await
                }
                Err(err) => IpcResponse::error(format!("Invalid IPC JSON payload: {}", err)),
            };

            let mut resp_bytes = serde_json::to_vec(&response)?;
            resp_bytes.push(b'\n');
            writer.write_all(&resp_bytes).await?;
            writer.flush().await?;
        }

        Ok(())
    }
}
