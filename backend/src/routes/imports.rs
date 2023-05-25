pub use crate::static_routes::extend::*;
pub use crate::{
    authentication::{reject_anonymous_users, validate_credentials, Credentials},
    database::*,
    error::{ApiError, ApiResult},
    startup::AppState,
    timeout::{HangingStrategy, TimeoutStrategy},
};
pub use anyhow::Context;
pub use axum::{
    extract::{
        rejection::{FormRejection, JsonRejection, TypedHeaderRejection},
        Extension, Form, Json, Path, Query, State, TypedHeader,
    },
    headers::{authorization::Basic, Authorization},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
pub use axum_sessions::extractors::{ReadableSession, WritableSession};
pub use common::interfacing;
pub use common::static_routes::*;
pub use secrecy::{ExposeSecret, SecretString};
pub use serde::{Deserialize, Serialize};

pub fn collect_contents<S>(docs: Vec<CollectionDocument<S>>) -> Vec<S::Contents>
where
    S: SerializedCollection,
{
    docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>()
}
