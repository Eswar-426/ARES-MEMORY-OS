use crate::engines::overview::models::VersionOverview;

pub async fn collect() -> VersionOverview {
    VersionOverview {
        ares_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: "1.0.0".to_string(),
        database_version: "SQLite 3".to_string(),
        extension_version: "0.2.0".to_string(),
    }
}
