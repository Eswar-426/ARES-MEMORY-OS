use crate::models::ContextPack;

pub trait ToJson {
    fn to_json(&self) -> String;
}

impl ToJson for ContextPack {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
