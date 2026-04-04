use serde::{Deserialize, Serialize};

/// Request sent by any client (CLI, app, extension)
#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub action: Action,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // Session
    Unlock,
    Lock,
    Status,

    // Entries
    ListEntries,
    GetEntry,
    AddEntry,
    UpdateEntry,
    DeleteEntry,

    // Generator
    GeneratePassword,
    CheckPassword,
}

/// Response sent back to the client
#[derive(Debug, Serialize, Default)]
pub struct Response {
    pub id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: impl Serialize) -> Self {
        Self {
            ok: true,
            data: Some(serde_json::to_value(data).unwrap_or(serde_json::Value::Null)),
            ..Default::default()
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(msg.into()),
            ..Default::default()
        }
    }
}
