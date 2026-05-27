//! MangoWM IPC service.
//!
//! Communicates with the Mango compositor over a Unix domain socket
//! specified by the `MANGO_INSTANCE_SIGNATURE` environment variable.

use std::{env, io, path::Path};

use futures::stream::BoxStream;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::warn;

/// One Mango tag (workspace equivalent in dwm-style compositors).
#[derive(Debug, Clone, Deserialize)]
pub struct MangoTag {
    /// Tag index (1-based).
    pub index: u32,
    /// Whether this tag is currently active (selected) on its monitor.
    pub is_active: bool,
    /// Whether any client on this tag is urgent.
    pub is_urgent: bool,
    /// Layout symbol for this tag.
    #[allow(dead_code)]
    pub layout: String,
    /// Number of clients on this tag.
    pub client_count: i32,
}

/// Response from `get tags <monitor>` or `watch tags <monitor>`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MangoTagsResponse {
    /// Monitor name.
    pub monitor: String,
    /// Tags on this monitor.
    pub tags: Vec<MangoTag>,
    /// Currently active tag indices on this monitor.
    pub active_tags: Vec<u32>,
}

/// A Mango client (window).
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MangoClient {
    /// Client id. `None` when no focused client.
    pub id: Option<u64>,
    /// Window title.
    pub title: Option<String>,
    /// Application id / class.
    #[serde(rename = "appid")]
    pub app_id: Option<String>,
    /// Monitor name.
    #[serde(default)]
    pub monitor: String,
    /// Tags this client is on.
    #[serde(default)]
    pub tags: Vec<u32>,
    /// Whether this client is focused.
    #[serde(default)]
    pub is_focused: bool,
    /// Whether this client is urgent.
    #[serde(default)]
    pub is_urgent: bool,
}

/// IPC client for the Mango compositor.
pub struct MangoService {
    socket_path: String,
}

impl MangoService {
    /// Creates a new Mango service if `MANGO_INSTANCE_SIGNATURE` is set.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is missing or the socket
    /// does not exist.
    pub fn new() -> io::Result<Self> {
        let socket_path = env::var("MANGO_INSTANCE_SIGNATURE").map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "MANGO_INSTANCE_SIGNATURE not set",
            )
        })?;

        if !Path::new(&socket_path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Mango socket not found: {socket_path}"),
            ));
        }

        Ok(Self { socket_path })
    }

    /// Returns the tags for a given monitor.
    pub async fn get_tags(&self, monitor: &str) -> io::Result<MangoTagsResponse> {
        let response = self.send_command(&format!("get tags {monitor}")).await?;
        Self::parse_json(&response)
    }

    /// Returns the currently focused client, if any.
    pub async fn get_focusing_client(&self) -> io::Result<Option<MangoClient>> {
        let response = self.send_command("get focusing-client").await?;
        let client: MangoClient = Self::parse_json(&response)?;
        Ok(client.id.map(|_| client))
    }

    /// Dispatches a Mango command.
    ///
    /// Example: `dispatch view,3` switches to tag 3.
    pub async fn dispatch(&self, cmd: &str) -> io::Result<()> {
        let response = self.send_command(&format!("dispatch {cmd}")).await?;
        if response.contains("error") {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Mango dispatch failed: {response}"),
            ));
        }
        Ok(())
    }

    /// Subscribes to tag changes for a monitor and returns a stream of
    /// [`MangoTagsResponse`] updates.
    pub fn watch_tags(&self, monitor: String) -> BoxStream<'static, io::Result<MangoTagsResponse>> {
        let socket_path = self.socket_path.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            if let Err(err) = watch_tags_inner(&socket_path, &monitor, tx).await {
                warn!(error = %err, "Mango tags watcher ended");
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }

    /// Subscribes to focusing-client changes and returns a stream of
    /// optional [`MangoClient`] updates.
    pub fn watch_focusing_client(&self) -> BoxStream<'static, io::Result<Option<MangoClient>>> {
        let socket_path = self.socket_path.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            if let Err(err) = watch_focusing_client_inner(&socket_path, tx).await {
                warn!(error = %err, "Mango focusing-client watcher ended");
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }

    // ------------------------------------------------------------------

    async fn send_command(&self, cmd: &str) -> io::Result<String> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        stream.write_all(format!("{cmd}\n").as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_to_string(&mut response).await?;
        Ok(response)
    }

    fn parse_json<T: for<'de> Deserialize<'de>>(text: &str) -> io::Result<T> {
        serde_json::from_str(text.trim()).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse Mango response: {err}"),
            )
        })
    }
}

async fn watch_tags_inner(
    socket_path: &str,
    monitor: &str,
    tx: mpsc::UnboundedSender<io::Result<MangoTagsResponse>>,
) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket_path).await?;
    stream
        .write_all(format!("watch tags {monitor}\n").as_bytes())
        .await?;

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let result = MangoService::parse_json::<MangoTagsResponse>(&line);
        if tx.send(result).is_err() {
            break;
        }
    }

    Ok(())
}

async fn watch_focusing_client_inner(
    socket_path: &str,
    tx: mpsc::UnboundedSender<io::Result<Option<MangoClient>>>,
) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket_path).await?;
    stream
        .write_all(b"watch focusing-client\n")
        .await?;

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let result = MangoService::parse_json::<MangoClient>(&line).map(|client| {
            if client.id.is_some() {
                Some(client)
            } else {
                None
            }
        });
        if tx.send(result).is_err() {
            break;
        }
    }

    Ok(())
}
