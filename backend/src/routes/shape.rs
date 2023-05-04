use crate::routes::imports::*;
use remote_database::shema::Shape;

#[axum_macros::debug_handler]
pub async fn all_shapes(Extension(shared_database): Extension<SharedRemoteDatabase>) -> Response {
    // Original solution to hanging client replaced by perpetural database pinging
    // Also a solution to any connection teardown
    // for example, to restore connection with a restarted database server
    async fn retry_maybe<F, C, R>(
        closure: C,
        shared_database: SharedRemoteDatabase,
    ) -> Result<R, ()>
    where
        F: std::future::Future<Output = R>,
        C: Fn() -> F,
    {
        let mut retried_times = 0;

        loop {
            match tokio::time::timeout_at(
                tokio::time::Instant::now() + std::time::Duration::from_secs(1),
                closure(),
            )
            .await
            {
                Ok(r) => return Ok(r),
                Err(_elapsed) => {
                    if retried_times >= 1 {
                        return Err(());
                    }

                    shared_database.write().await.reconfigure().await;
                    retried_times += 1;
                }
            }
        }
    }

    tracing::info!("Remote database ID: {}", shared_database.read().await.id);

    let shapes = shared_database.read().await.collections.shapes.clone();

    let docs = retry_maybe(|| Shape::all_async(&shapes), shared_database.clone())
        .await
        .expect("must connect")
        .expect("must fetch");

    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
    return Json(contents).into_response();

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
