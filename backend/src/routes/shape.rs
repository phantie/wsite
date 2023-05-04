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
    // Original solution to hanging client replaced by perpetural database pinging
    // Also a solution to any connection teardown
    // for example, to restore connection with a restarted database server
    async fn execute<F, C, R>(
        self,
        closure: C,
        shared_database: SharedRemoteDatabase,
    ) -> Result<R, ()>
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
                                return Err(());
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
) -> impl IntoResponse {
    tracing::info!("Remote database ID: {}", shared_database.read().await.id);

    let shapes = shared_database.read().await.collections.shapes.clone();

    let docs = HangingStrategy::default()
        .execute(|| Shape::all_async(&shapes), shared_database.clone())
        .await
        .expect("failed to unhang")
        .expect("failed to fetch valid data");

    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();

    Json(contents)

    // let shapes = state.remote_database.collections.shapes;
    // let docs = Shape::all_async(&shapes).await.unwrap();
    // let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
    // Json(contents).into_response()
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
