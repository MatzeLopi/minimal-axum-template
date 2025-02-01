use crate::{
    crud,
    http::{dependencies, error::Error as HTTPError, AppState},
    schemas::users::NewUser,
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
        .route("/users/delete-user", delete(delete_user))
        .route("/users/me", get(me))
        .route("/users/me/update-password", post(update_password))
        .route("/users/available/username", post(username_available))
        .route("/users/available/email", post(email_available))
        .route("/users/verify/{username}/{token}", get(verify_user))
        .with_state(state)
}

async fn me(
    State(state): State<Arc<AppState>>,
    _: dependencies::CsrfValidator,
    auth_user: dependencies::AuthUser,
) -> Result<impl IntoResponse, HTTPError> {
    let user_id = auth_user.user_id;
    let user = crud::user::get_user_by_id(&user_id, &state.db).await?;
    Ok((StatusCode::OK, Json(user)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<NewUser>,
) -> Result<impl IntoResponse, HTTPError> {
    log::debug!("New user creation started");
    let NewUser {
        username,
        email,
        password,
    } = user;

    let password_hash = dependencies::hash_password(password)?;

    _ = crud::user::create_user(&username, &email, &password_hash, state).await?;
    log::debug!("Successfully created new user");
    Ok((StatusCode::CREATED, "User created successfully"))
}

async fn update_password(
    State(state): State<Arc<AppState>>,
    _: dependencies::CsrfValidator,
    user: dependencies::AuthUser,
    password: String,
) -> Result<impl IntoResponse, HTTPError> {
    let pw_hash = dependencies::hash_password(password)?;
    if crud::user::update_password(&user.user_id, &pw_hash, &state.db).await? == true {
        Ok(StatusCode::OK)
    } else {
        log::error!("Failed to update password");
        Err(HTTPError::InternalServerError)
    }
}

async fn username_available(
    State(state): State<Arc<AppState>>,
    username: String,
) -> Result<impl IntoResponse, HTTPError> {
    match crud::user::check_username(&username, &state.db).await {
        true => {
            log::debug!("Username is available");
            Ok(StatusCode::OK)
        }
        false => Ok({
            log::debug!("Username is taken");
            StatusCode::CONFLICT
        }),
    }
}

async fn email_available(
    State(state): State<Arc<AppState>>,
    email: String,
) -> Result<impl IntoResponse, HTTPError> {
    match crud::user::check_email(&email, &state.db).await {
        true => {
            log::debug!("Email is available");
            Ok(StatusCode::OK)
        }
        false => {
            log::debug!("Email is taken");
            Ok(StatusCode::CONFLICT)
        }
    }
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
    _: dependencies::CsrfValidator,
    auth_user: dependencies::AuthUser,
) -> Result<impl IntoResponse, HTTPError> {
    _ = crud::user::delete_user(&auth_user.user_id, &state.db).await?;
    Ok((StatusCode::OK, "Successfully deleted user"))
}
