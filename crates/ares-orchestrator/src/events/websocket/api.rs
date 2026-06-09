use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use super::connection::WsConnection;
use std::sync::Arc;

pub struct WsApiState {
    pub hub: Arc<super::hub::WsHub>,
}

pub fn router() -> Router<Arc<WsApiState>> {
    Router::new().route("/ws", get(ws_handler))
}

async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<Arc<WsApiState>>) -> impl IntoResponse {
    ws.on_upgrade(WsConnection::handle)
}
