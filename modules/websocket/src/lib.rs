use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use tails_native_macros::{tails_function, tails_module};

// ============================================================================
// Public API for direct Rust usage
// ============================================================================

/// Lazily-initialized tokio runtime used to drive async WebSocket operations
/// from synchronous FFI callbacks.
static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .expect("failed to create tails-websocket tokio runtime")
});

pub struct WebSocket {
    inner: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    url: String,
}

impl WebSocket {
    pub fn new(url: &str) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            url: url.to_string(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn connect(&self) -> Result<(), String> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        *self.inner.lock().await = Some(ws_stream);
        Ok(())
    }

    pub async fn send(&self, message: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        if let Some(ws) = inner.as_mut() {
            ws.send(tokio_tungstenite::tungstenite::Message::Text(
                message.to_string(),
            ))
            .await
            .map_err(|e| format!("Send failed: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    pub async fn receive(&self) -> Result<String, String> {
        let mut inner = self.inner.lock().await;
        if let Some(ws) = inner.as_mut() {
            match ws.next().await {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => Ok(text),
                Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) => {
                    Ok(String::from_utf8_lossy(&data).to_string())
                }
                Some(Ok(_)) => Ok(String::new()),
                Some(Err(e)) => Err(format!("Receive error: {}", e)),
                None => Err("Connection closed".to_string()),
            }
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    pub async fn close(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        if let Some(mut ws) = inner.take() {
            ws.close(None)
                .await
                .map_err(|e| format!("Close failed: {}", e))?;
        }
        Ok(())
    }
}

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

/// Synchronous bridge — drives an async future to completion on a shared
/// tokio runtime. Returns a JSON string with either `{ok: true}` or
/// `{ok: false, error: "..."}`.
fn block_on_json<F>(fut: F) -> String
where
    F: std::future::Future<Output = Result<(), String>> + Send + 'static,
{
    let result = RUNTIME.block_on(fut);
    match result {
        Ok(()) => serde_json::json!({ "ok": true }).to_string(),
        Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
    }
}

#[tails_module(name = "tails-websocket")]
mod websocket_native {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Per-instance state table, keyed by the id assigned to the JS handle.
    static INSTANCES: Lazy<Mutex<std::collections::HashMap<u32, Arc<WebSocket>>>> =
        Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

    /// Allocates a new opaque handle id. The `__tails_` prefix prevents the
    /// module macro from exporting this as a JS function.
    fn __tails_next_id() -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    #[tails_function]
    pub fn create(url: String) -> f64 {
        let id = __tails_next_id();
        let ws = Arc::new(WebSocket::new(&url));
        RUNTIME.block_on(async {
            let mut map = INSTANCES.lock().await;
            map.insert(id, ws);
        });
        id as f64
    }

    #[tails_function]
    pub fn url(id: f64) -> String {
        let id = id as u32;
        let ws = RUNTIME.block_on(async {
            let map = INSTANCES.lock().await;
            map.get(&id).cloned()
        });
        ws.map(|w| w.url().to_string()).unwrap_or_default()
    }

    #[tails_function]
    pub fn connect(id: f64) -> String {
        let id = id as u32;
        let ws = RUNTIME.block_on(async {
            let map = INSTANCES.lock().await;
            map.get(&id).cloned()
        });
        match ws {
            Some(ws) => block_on_json(async move { ws.connect().await }),
            None => serde_json::json!({ "ok": false, "error": "invalid handle" }).to_string(),
        }
    }

    #[tails_function]
    pub fn send(id: f64, message: String) -> String {
        let id = id as u32;
        let ws = RUNTIME.block_on(async {
            let map = INSTANCES.lock().await;
            map.get(&id).cloned()
        });
        match ws {
            Some(ws) => block_on_json(async move { ws.send(&message).await }),
            None => serde_json::json!({ "ok": false, "error": "invalid handle" }).to_string(),
        }
    }

    #[tails_function]
    pub fn receive(id: f64) -> String {
        let id = id as u32;
        let ws = RUNTIME.block_on(async {
            let map = INSTANCES.lock().await;
            map.get(&id).cloned()
        });
        match ws {
            Some(ws) => {
                let res = RUNTIME.block_on(async move { ws.receive().await });
                match res {
                    Ok(text) => serde_json::json!({ "ok": true, "data": text }).to_string(),
                    Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
                }
            }
            None => serde_json::json!({ "ok": false, "error": "invalid handle" }).to_string(),
        }
    }

    #[tails_function]
    pub fn close(id: f64) -> String {
        let id = id as u32;
        let ws = RUNTIME.block_on(async {
            let map = INSTANCES.lock().await;
            map.get(&id).cloned()
        });
        match ws {
            Some(ws) => block_on_json(async move { ws.close().await }),
            None => serde_json::json!({ "ok": false, "error": "invalid handle" }).to_string(),
        }
    }

    #[tails_function]
    pub fn destroy(id: f64) -> bool {
        let id = id as u32;
        RUNTIME.block_on(async {
            let mut map = INSTANCES.lock().await;
            map.remove(&id).is_some()
        })
    }
}
