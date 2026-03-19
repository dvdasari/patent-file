use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ISS_ACCESS: &str = "pdp:access";
pub const ISS_REFRESH: &str = "pdp:refresh";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iss: String,
}

impl Claims {
    pub fn user_id(&self) -> anyhow::Result<Uuid> {
        Uuid::parse_str(&self.sub)
            .map_err(|e| anyhow::anyhow!("Invalid user ID in token: {}", e))
    }

    pub fn is_access_token(&self) -> bool {
        self.iss == ISS_ACCESS
    }

    pub fn is_refresh_token(&self) -> bool {
        self.iss == ISS_REFRESH
    }
}

pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Issue a JWT access token (24h expiry)
pub fn issue_access_token(user_id: &Uuid, secret: &str) -> anyhow::Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now().timestamp() + 86400) as usize,
        iss: ISS_ACCESS.to_string(),
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

/// Issue a refresh token (7 day expiry)
pub fn issue_refresh_token(user_id: &Uuid, secret: &str) -> anyhow::Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now().timestamp() + 604800) as usize,
        iss: ISS_REFRESH.to_string(),
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

/// Hash a refresh token for storage (SHA-256)
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(token.as_bytes());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash)
}

pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            request
                .headers()
                .get("Cookie")
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find_map(|c| c.trim().strip_prefix("access_token="))
                        .map(|s| s.to_string())
                })
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims =
        validate_token(&token, &jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !claims.is_access_token() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id = claims.user_id().map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(AuthUser { user_id });
    Ok(next.run(request).await)
}

#[cfg(test)]
pub fn create_test_token(secret: &str, sub: &str, expires_in_secs: i64) -> String {
    create_test_token_with_iss(secret, sub, expires_in_secs, ISS_ACCESS)
}

#[cfg(test)]
pub fn create_test_token_with_iss(
    secret: &str,
    sub: &str,
    expires_in_secs: i64,
    iss: &str,
) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = Claims {
        sub: sub.to_string(),
        exp: (chrono::Utc::now().timestamp() + expires_in_secs) as usize,
        iss: iss.to_string(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id_from_valid_claims() {
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now().timestamp() as usize + 3600,
            iss: ISS_ACCESS.to_string(),
        };
        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_reject_expired_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = create_test_token(secret, &user_id.to_string(), -3600);
        assert!(validate_token(&token, secret).is_err());
    }

    #[test]
    fn test_accept_valid_token() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let token = create_test_token(secret, &user_id.to_string(), 3600);
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);
        assert!(claims.is_access_token());
    }

    #[test]
    fn test_reject_wrong_secret() {
        let user_id = Uuid::new_v4();
        let token = create_test_token("secret-a", &user_id.to_string(), 3600);
        assert!(validate_token(&token, "secret-b").is_err());
    }

    #[test]
    fn test_token_type_differentiation() {
        let secret = "test-secret";
        let user_id = Uuid::new_v4();
        let access = create_test_token_with_iss(secret, &user_id.to_string(), 3600, ISS_ACCESS);
        let refresh =
            create_test_token_with_iss(secret, &user_id.to_string(), 3600, ISS_REFRESH);

        let access_claims = validate_token(&access, secret).unwrap();
        assert!(access_claims.is_access_token());
        assert!(!access_claims.is_refresh_token());

        let refresh_claims = validate_token(&refresh, secret).unwrap();
        assert!(!refresh_claims.is_access_token());
        assert!(refresh_claims.is_refresh_token());
    }

    #[test]
    fn test_hash_token_deterministic() {
        let hash1 = hash_token("my-token");
        let hash2 = hash_token("my-token");
        assert_eq!(hash1, hash2);
        assert_ne!(hash_token("other-token"), hash1);
    }
}
