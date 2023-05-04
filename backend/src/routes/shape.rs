use crate::routes::imports::*;
use remote_database::shema::Shape;
use std::time::Duration;

enum HangingStrategy {
    #[allow(dead_code)]
    LinearRetry { times: u32, sleep: Duration },
}

impl Default for HangingStrategy {
    fn default() -> Self {
        Self::LinearRetry {
            times: 1,
            sleep: Duration::from_secs(1),
        }
    }
}

impl HangingStrategy {
    // Attempt to renew connection with the database server if it hangs
    // because there's no timeouts on external API calls to it
    async fn execute<F, C, R>(
        self,
        closure: C,
        shared_database: SharedRemoteDatabase,
    ) -> Result<R, ApiError>
    where
        F: std::future::Future<Output = R>,
        C: Fn() -> F,
    {
        match self {
            Self::LinearRetry {
                times: max_times,
                sleep,
            } => {
                let mut retried_times = 0;

                loop {
                    match tokio::time::timeout_at(tokio::time::Instant::now() + sleep, closure())
                        .await
                    {
                        Ok(r) => return Ok(r),
                        Err(_elapsed) => {
                            if retried_times >= max_times {
                                tracing::info!("FUCKING ERROR");
                                return Err(ApiError::DatabaseHangs);
                            }

                            shared_database.write().await.reconfigure().await;
                            retried_times += 1;
                        }
                    }
                }
            }
        }
    }
}

#[axum_macros::debug_handler]
pub async fn all_shapes(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> Result<Json<Vec<Shape>>, ApiError> {
    tracing::info!("Remote database ID: {}", shared_database.read().await.id);

    // BUG after shared database is updated it still uses the old ref, &shapes in this case
    let shapes = shared_database.read().await.collections.shapes.clone();

    let docs = HangingStrategy::default()
        .execute(|| Shape::all_async(&shapes), shared_database.clone())
        .await?
        .expect("failed to fetch valid data");

    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();

    Ok(Json(contents))
}

#[axum_macros::debug_handler]
pub async fn new_shape(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(body): Json<Shape>,
) -> Response {
    let shapes = shared_database.read().await.collections.shapes.clone();
    body.push_into_async(&shapes).await.unwrap();
    StatusCode::OK.into_response()
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Json is rejected")]
    JsonRejection(#[from] JsonRejection),

    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error("Database hangs")]
    DatabaseHangs,

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
        }
        .into_response()
    }
}
