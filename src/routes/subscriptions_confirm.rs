use axum::extract::Query;
use axum::http::StatusCode;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm(Query(parameters): Query<Parameters>) -> StatusCode {
    StatusCode::OK
}
