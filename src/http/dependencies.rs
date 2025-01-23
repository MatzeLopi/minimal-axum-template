// External Crates
use crate::crud;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{
        header::{HeaderValue, AUTHORIZATION},
        request::Parts,
    },
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

// Internal Modules
use crate::http::{error::Error as HTTPError, AppState};

const DEFAULT_SESSION_DURATION: time::Duration = time::Duration::weeks(1);

const DEFAULT_AUTH: &str = "JWT";

pub struct AuthUser {
    pub user_id: Uuid,
}

// Use in handler if auth is optional
pub struct OptionalAuthUser(pub Option<AuthUser>);

pub struct CsrfValidator;

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthClaims {
    sub: Uuid,
    exp: i64,
}

fn hash_password(password: String) -> Result<String, HTTPError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => {
            log::debug!("Failed to hash password: {:?}", e);
            Err(HTTPError::InternalServerError)
        }
    }
}

fn validate_password(password: &str, password_hash: &str) -> Result<bool, HTTPError> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
        log::debug!("Invalid password hash format: {:?}", e);
        HTTPError::Unauthorized
    })?;
    let result = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    match result {
        true => Ok(true),
        false => Err(HTTPError::Unauthorized),
    }
}

pub async fn auth_user(username: &str, password: &str, db: &PgPool) -> Result<AuthUser, HTTPError> {
    // Fetch password hash from the database
    let (id, password_hash) = crud::user::get_hash(username, db).await?;

    // Validate the password
    validate_password(&password, &password_hash)?;

    Ok(AuthUser { user_id: id })
}

impl AuthUser {
    pub(in crate::http) fn to_jwt(&self, context: &AppState) -> Result<String, HTTPError> {
        let secret = &context.config.hmac_key;
        let token = encode(
            &Header::default(),
            &AuthClaims {
                sub: self.user_id,
                exp: (OffsetDateTime::now_utc() + DEFAULT_SESSION_DURATION).unix_timestamp(),
            },
            &EncodingKey::from_secret(secret.as_ref()),
        );

        match token {
            Ok(token) => Ok(token),
            Err(e) => {
                log::debug!("Failed to encode token: {:?}", e);
                Err(HTTPError::InternalServerError)
            }
        }
    }
    fn from_authorization(ctx: &AppState, auth_header: &HeaderValue) -> Result<Self, HTTPError> {
        let auth_header = auth_header.to_str().map_err(|_| {
            log::debug!("Authorization header is not UTF-8");
            HTTPError::Unauthorized
        })?;

        if !auth_header.starts_with(DEFAULT_AUTH) {
            log::debug!(
                "Authorization header is using the wrong scheme: {:?}",
                auth_header
            );
            return Err(HTTPError::Unauthorized);
        }

        let jwt_token = &auth_header[DEFAULT_AUTH.len()..];
        let secret = &ctx.config.hmac_key;
        // `token` is a struct with 2 fields: `header` and `claims` where `claims` is your own struct.
        let token = decode::<AuthClaims>(
            &jwt_token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| {
            log::debug!("Failed to decode token: {:?}", e);
            HTTPError::Unauthorized
        });

        match token {
            Ok(token) => Ok(AuthUser {
                user_id: token.claims.sub,
            }),
            Err(e) => Err(e),
        }
    }
}

impl OptionalAuthUser {
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|auth_user| auth_user.user_id)
    }
}

impl<S> FromRequestParts<S> for CsrfValidator
where
    S: Send + Sync,
{
    type Rejection = HTTPError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let client_token = parts.headers.get("X-CSRF-TOKEN");
        let server_token = parts.headers.get("csrf_token"); // Ensure this is properly added to the request

        // Validate that both tokens exist and are equal
        match (client_token, server_token) {
            (Some(client_token), Some(server_token)) if client_token == server_token => Ok(Self),
            _ => Err(HTTPError::Unauthorized),
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = HTTPError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the `ApiContext` extension
        let ctx = state.as_ref();

        // Get the `Authorization` header
        let auth_header = parts.headers.get(AUTHORIZATION).ok_or_else(|| {
            log::debug!("Authorization header is missing");
            HTTPError::Unauthorized
        })?;

        // Process the Authorization header
        AuthUser::from_authorization(ctx, auth_header)
    }
}

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = HTTPError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the `ApiContext` extension
        let ctx = state.as_ref();

        // Check if the `Authorization` header exists
        let auth_user = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|auth_header| AuthUser::from_authorization(ctx, auth_header).ok());

        Ok(Self(auth_user))
    }
}
