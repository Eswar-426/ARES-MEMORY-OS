use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    if std::env::var("ARES_AUTH_DISABLED")
        .unwrap_or_default()
        .eq_ignore_ascii_case("true")
    {
        return Ok(next.run(req).await);
    }

    // Basic API Key validation
    // For this local platform, we just check if an API key is present.
    // In a real scenario, this would validate against the database or config.
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                // Let's assume all Bearer tokens are valid Developer/Admin for now
                // Future: Validate against db/config, attach role to request extensions
                return Ok(next.run(req).await);
            }
        }
    }

    // Reject if no valid auth provided
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    if std::env::var("ARES_AUTH_DISABLED")
        .unwrap_or_default()
        .eq_ignore_ascii_case("true")
    {
        return Ok(next.run(req).await);
    }

    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str == "Bearer admin-token" {
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::FORBIDDEN)
}
