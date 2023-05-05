use crate::routes::imports::*;
use remote_database::shema::Shape;
use std::{sync::Arc, time::Duration};

// Timeouts for closures that don't need to change state when retrying
pub enum TimeoutStrategy {
    #[allow(dead_code)]
    Once { timeout: Duration },
}

impl Default for TimeoutStrategy {
    fn default() -> Self {
        Self::Once {
            timeout: Duration::from_secs(3),
        }
    }
}

impl TimeoutStrategy {
    pub async fn execute<F, C, R>(self, closure: C) -> Result<R, ApiError>
    where
        C: Fn() -> F,
        F: std::future::Future<Output = R>,
    {
        match self {
            Self::Once { timeout } => {
                match tokio::time::timeout_at(tokio::time::Instant::now() + timeout, closure())
                    .await
                {
                    Ok(v) => Ok(v),
                    Err(_) => Err(ApiError::FutureTimeout),
                }
            }
        }
    }
}

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
        C: Fn(SharedRemoteDatabase) -> F,
        F: std::future::Future<Output = R>,
    {
        match self {
            Self::LinearRetry {
                times: max_times,
                sleep,
            } => {
                let mut retried_times = 0;

                loop {
                    let id = shared_database.read().await.id;
                    match tokio::time::timeout_at(
                        tokio::time::Instant::now() + sleep,
                        closure(Arc::clone(&shared_database)),
                    )
                    .await
                    {
                        Ok(r) => return Ok(r),
                        Err(_elapsed) => {
                            if retried_times >= max_times {
                                return Err(ApiError::DatabaseHangs);
                            }

                            // When several requests hang - reconfigure the client once, let others wait
                            // Before deciding whether to reconfigure client -
                            // check the id of the client the request was tried with
                            // if ID does not match - client has changed, so retry the request
                            {
                                let mut shared_database = shared_database.write().await;
                                if shared_database.id == id {
                                    tracing::info!(
                                        "Reconfiguring... remote database client ID: {}",
                                        shared_database.id
                                    );

                                    if let Ok(()) = shared_database.reconfigure().await {
                                        tracing::info!(
                                            "Reconfigured remote database client ID: {}",
                                            shared_database.id
                                        );
                                    }

                                    retried_times += 1;
                                }
                            }
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

    let docs = HangingStrategy::default()
        .execute(
            |shared_database| async move {
                let shapes = &shared_database.read().await.collections.shapes;
                Shape::all_async(shapes).await
            },
            shared_database.clone(),
        )
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
