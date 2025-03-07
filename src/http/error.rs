use axum::http::header::WWW_AUTHENTICATE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("authentication required")]
    Unauthorized,

    #[error("user may not perform that action")]
    Forbidden,

    #[error("request path not found")]
    NotFound,

    #[error("Internal Server Error")]
    InternalServerError,

    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),

    #[error("conflict, resource already exists")]
    Conflict,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Conflict => StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    [(WWW_AUTHENTICATE, "JWT")],
                    self.to_string(),
                )
                    .into_response();
            }

            Self::Sqlx(ref e) => {
                log::error!("SQLx error: {:?}", e);
            }

            Self::Anyhow(ref e) => {
                log::error!("Generic error: {:?}", e);
            }

            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
