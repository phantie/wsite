#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Json is rejected")]
    JsonRejection(#[from] JsonRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error("Database hangs")]
    DatabaseHangs,

    #[error("Future timeout")]
    FutureTimeout,

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
            _ => self.to_string(),
        };
        tracing::error!("{}", trace_message);

        match &self {
            Self::JsonRejection(_e) => StatusCode::BAD_REQUEST,
            Self::AuthError(_e) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseHangs => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FutureTimeout => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

use axum::{
    extract::rejection::JsonRejection,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
