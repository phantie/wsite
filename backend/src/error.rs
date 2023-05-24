#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Json is rejected")]
    JsonRejection(#[from] JsonRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error("Database hangs")]
    DatabaseHangs,

    #[error("Entry not found")]
    EntryNotFound,

    #[error("Bad request")]
    BadRequest,

    #[error("Database error: {0}")]
    DatabaseError(#[from] bonsaidb::core::Error),

    // #[error("Database insert error: {0}")]
    // DatabaseInsertError(#[source] anyhow::Error),
    //
    #[error("Future timeout")]
    FutureTimeout,

    #[error("Auth header rejected")]
    AuthHeaderRejection(#[source] TypedHeaderRejection),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let trace_message = match &self {
            Self::JsonRejection(rejection) => {
                format!("{}: {}", self.to_string(), rejection.to_string())
            }
            Self::UnexpectedError(e) => format!("{}: {}", self.to_string(), e.source().unwrap()),
            Self::AuthError(e) => format!("{}: {}", self.to_string(), e.source().unwrap()),
            _ => self.to_string(),
        };
        tracing::error!("{}", trace_message);

        match &self {
            Self::JsonRejection(_e) => StatusCode::BAD_REQUEST,
            Self::AuthError(_e) => StatusCode::UNAUTHORIZED,
            Self::AuthHeaderRejection(_e) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::EntryNotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::DatabaseHangs => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            // Self::DatabaseInsertError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FutureTimeout => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

use axum::{
    extract::rejection::{JsonRejection, TypedHeaderRejection},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
