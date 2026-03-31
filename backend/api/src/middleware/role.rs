use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

use super::auth::{AuthUser, AuthUserWithRole, Role};

/// Middleware that loads the user's role from the database and inserts
/// `AuthUserWithRole` into request extensions.
/// Must run AFTER `auth_middleware` (requires `AuthUser` in extensions).
pub async fn role_middleware(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let role_str: String = sqlx::query_scalar("SELECT role::text FROM users WHERE id = $1")
        .bind(user.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let role = Role::from_str(&role_str).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    request.extensions_mut().insert(AuthUserWithRole {
        user_id: user.user_id,
        role,
    });

    Ok(next.run(request).await)
}

/// Create a middleware that requires a specific set of roles.
/// Returns 403 Forbidden if the user's role is not in the allowed list.
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<AuthUserWithRole>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Require patent_agent or admin role.
pub async fn require_patent_agent_or_admin(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = request
        .extensions()
        .get::<AuthUserWithRole>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if user.role != Role::PatentAgent && user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
