// Router for auth and csrf token generation
use crate::{
    http::{dependencies, error::Error as HTTPError, utils, AppState},
    schemas::users::UserLogin,
};

use axum_extra::extract::cookie::{Cookie, CookieJar, Expiration};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, Router},
};
use serde_json::json;
use std::sync::Arc;

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

async fn get_csfr(mut jar: CookieJar) -> impl IntoResponse {
    // Generate a random token
    let csft: String = utils::random_string(32);
    let expiration = Expiration::from(time::OffsetDateTime::now_utc() + time::Duration::hours(24));
    let server_cookie = Cookie::build(("s_csft", csft.clone()))
        .secure(true)
        .http_only(true)
        .expires(expiration)
        .build();
    let client_cookie = Cookie::build(("x_csft", csft))
        .secure(true)
        .http_only(false)
        .expires(expiration)
        .build();
    jar = jar
        .remove(Cookie::from("s_csft"))
        .remove(Cookie::from("x_csft"))
        .add(server_cookie)
        .add(client_cookie);

    log::debug!("Request");
    return (StatusCode::OK, jar);
}

async fn token(
    State(state): State<Arc<AppState>>,
    _: dependencies::CsrfValidator,
    Json(user): Json<UserLogin>,
) -> Result<impl IntoResponse, HTTPError> {
    let db = &state.db;
    let UserLogin { username, password } = user;
    let auth_user = dependencies::auth_user(&username, password, db).await;
    match auth_user {
        Ok(auth_user) => {
            let token = auth_user.to_jwt(&state)?;
            Ok((StatusCode::OK, Json(token)))
        }
        Err(e) => Err(e),
    }
}
