use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::Deserialize;

use super::jwt::decode_token;
use crate::auth::config::AuthConfig;

/// Authenticated user extracted from JWT in either:
/// 1. Authorization: Bearer <token> header
/// 2. ?token=<token> query parameter
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub is_admin: bool,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + AsRef<AuthConfig>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = state.as_ref();

        // Try Authorization: Bearer <token> header first
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        // Fall back to ?token= query parameter
        let token = match token {
            Some(t) => t,
            None => {
                let Query(q) = Query::<TokenQuery>::try_from_uri(&parts.uri)
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing authentication token"))?;
                q.token
                    .ok_or((StatusCode::UNAUTHORIZED, "Missing authentication token"))?
            }
        };

        let claims = decode_token(&token, &config.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
            is_admin: claims.is_admin,
        })
    }
}
