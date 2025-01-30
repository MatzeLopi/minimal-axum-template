use crate::{
    crud,
    http::{dependencies, error::Error as HTTPError, AppState},
    schemas::users::{NewUser, User},
};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, Router},
};
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/create-user", post(create_user))
        .route("/delete-user", delete(delete_user))
        .route("/me", get(me))
        .route("/auth-user/:username/:token", get(verify_user))
        .with_state(state)
}

async fn me(
    State(state): State<Arc<AppState>>,
    option_user: dependencies::OptionalAuthUser,
) -> Result<impl IntoResponse, HTTPError> {
    let user_id = option_user.user_id();

    let user = match user_id {
        Some(user_id) => crud::user::get_user_by_id(&user_id, &state.db).await?,
        None => return Ok((StatusCode::OK, Json(User::default()))),
    };
    Ok((StatusCode::OK, Json(user)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<NewUser>,
) -> Result<impl IntoResponse, HTTPError> {
    let NewUser {
        username,
        email,
        password,
    } = user;
    let password_hash = dependencies::hash_password(password)?;

    _ = crud::user::create_user(&username, &email, &password_hash, state).await?;

    Ok((StatusCode::CREATED, "User created successfully"))
}

async fn verify_user(
    State(state): State<Arc<AppState>>,
    Path((username, token)): Path<(String, String)>,
) -> Result<impl IntoResponse, HTTPError> {
    if crud::user::get_verification_token(&username, &state.db).await? == token {
        _ = crud::user::verify_user(&username, &state.db).await?;
        Ok((StatusCode::OK, "User successfully verified"))
    } else {
        Err(HTTPError::Forbidden)
    }
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    auth_user: dependencies::AuthUser,
) -> Result<impl IntoResponse, HTTPError> {
    _ = crud::user::delete_user(&auth_user.user_id, &state.db).await?;
    Ok((StatusCode::OK, "Successfully deleted user"))
}
