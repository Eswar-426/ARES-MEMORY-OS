use serde::{Deserialize, Serialize};

/// IPC request envelope — newline-delimited JSON over socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IpcRequest {
    /// Client-generated correlation ID
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// IPC response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IpcResponse {
    /// Matches the request id
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<IpcError>,
}

impl IpcResponse {
    #[allow(dead_code)]
    pub fn ok(id: String, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn err(id: String, code: i32, message: String) -> Self {
        Self {
            id,
            result: None,
            error: Some(IpcError {
                code,
                message,
                details: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
