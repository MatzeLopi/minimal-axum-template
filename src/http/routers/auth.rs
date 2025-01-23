// Router for auth and csrf token generation
use crate::{
    http::{dependencies, error::Error as HTTPError, utils, AppState},
    schemas::users::UserLogin,
};

use axum::{
    extract::{Json, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, Router},
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(ok))
        .route("/csft", get(get_csfr))
        .route("/token", post(token))
        .with_state(state)
}

async fn ok() -> impl IntoResponse {
    (StatusCode::OK, axum::Json(json!({ "status": "ok" })))
}

async fn get_csfr() -> impl IntoResponse {
    // Generate a random token
    let csrf_token: String = utils::random_string(32);

    // Cookie
    let cookie: String = format!(
        "csrf_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}",
        csrf_token,
        Duration::from_secs(60 * 60 * 24).as_secs() // 1 day
    );

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    return (
        StatusCode::OK,
        headers,
        format!(r#"{{"csrf_token": "{}"}}"#, csrf_token),
    );
}

async fn token(
    State(state): State<Arc<AppState>>,
    Json(user): Json<UserLogin>,
) -> Result<impl IntoResponse, HTTPError> {
    let db = &state.db;
    let auth_user = dependencies::auth_user(&user.username, &user.password, db).await;
    match auth_user {
        Ok(auth_user) => {
            let token = auth_user.to_jwt(&state)?;
            Ok(token)
        }
        Err(e) => Err(e),
    }
}
