#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Json is rejected: {0}")]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error("Entry not found")]
    EntryNotFound,

    #[error("Bad request")]
    BadRequest,

    #[error("Database error: {0}")]
    DbError(#[from] crate::db::Error),

    #[error("Future timeout")]
    FutureTimeout,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let trace_message = match &self {
            Self::AuthError(e) => format!("{}: {}", self, e.root_cause()),
            Self::DbError(e) => format!("{:?}", e),
            _ => self.to_string(),
        };
        tracing::error!("{}", trace_message);

        use hyper::StatusCode;
        match &self {
            Self::JsonRejection(_e) => StatusCode::BAD_REQUEST,
            Self::AuthError(_e) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::EntryNotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::DbError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            // Self::DatabaseInsertError(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FutureTimeout => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
