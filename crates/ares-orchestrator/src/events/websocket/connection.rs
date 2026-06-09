use axum::extract::ws::WebSocket;
use futures::stream::StreamExt;

pub struct WsConnection;

impl WsConnection {
    pub async fn handle(mut socket: WebSocket) {
        // Echo loop as a basic placeholder. 
        // In reality, this would hook into the `hub.rs` to broadcast events.
        while let Some(msg) = socket.next().await {
            if let Ok(msg) = msg {
                if socket.send(msg).await.is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
