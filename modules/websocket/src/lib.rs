use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

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
