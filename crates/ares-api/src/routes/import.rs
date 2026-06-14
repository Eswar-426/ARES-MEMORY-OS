use crate::AppState;
use axum::routing::post;
use axum::Router;
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new().route("/import", post(import_chat))
}

async fn import_chat(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    let mut raw_content = String::new();
    let mut project_id = String::new();
    let mut format = "json_generic".to_string();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "file" {
            let data = field.bytes().await.unwrap();
            raw_content = String::from_utf8_lossy(&data).to_string();
        } else if name == "project_id" {
            project_id = field.text().await.unwrap();
        } else if name == "format" {
            format = field.text().await.unwrap();
        }
    }

    if raw_content.is_empty() || project_id.is_empty() {
        return Json(json!({ "error": "Missing file or project_id" }));
    }

    use ares_chat_import::types::{ChatFormat, ImportContext};

    let format_enum = match format.to_lowercase().as_str() {
        "chatgpt" => ChatFormat::ChatGPT,
        "claude" => ChatFormat::Claude,
        "gemini" => ChatFormat::Gemini,
        "cursor" => ChatFormat::Cursor,
        "markdown" => ChatFormat::Markdown,
        _ => ChatFormat::JsonGeneric,
    };

    let context = ImportContext {
        project_id,
        format: format_enum,
    };

    let pipeline = ares_chat_import::pipeline::ImportPipeline::new(
        state.memory_repo.clone(),
        state.graph_repo.clone(),
    );

    match pipeline.process_import(&raw_content, &context).await {
        Ok(count) => Json(json!({ "success": true, "imported_entities": count })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
