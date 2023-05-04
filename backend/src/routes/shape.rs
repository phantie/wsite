use crate::routes::imports::*;
use remote_database::shema::Shape;

#[axum_macros::debug_handler]
#[allow(unused_mut)]
pub async fn all_shapes(Extension(shared_database): Extension<SharedRemoteDatabase>) -> Response {
    // async fn retry_maybe<F>(future: F, remote_database: &mut RemoteDatabase) -> Result<(), ()>
    // where
    //     F: std::future::Future + Clone,
    // {
    //     let mut retried_times = 0;

    //     loop {
    //         match tokio::time::timeout_at(
    //             tokio::time::Instant::now() + std::time::Duration::from_secs(2),
    //             future.clone(),
    //         )
    //         .await
    //         {
    //             Ok(_) => return Ok(()),
    //             Err(_) => {
    //                 if retried_times >= 1 {
    //                     return Err(());
    //                 }

    //                 remote_database.reconfigure().await;
    //                 retried_times += 1;
    //             }
    //         }
    //     }
    // }

    tracing::info!("Remote database ID: {}", shared_database.read().await.id);

    // Original solution to hanging client replaced by perpetural database pinging
    // Also a solution to any connection teardown
    // for example, to restore connection with a restarted database server
    let mut retried = false;
    loop {
        let shapes = shared_database.read().await.collections.shapes.clone();

        match tokio::time::timeout_at(
            tokio::time::Instant::now() + std::time::Duration::from_secs(2),
            Shape::all_async(&shapes),
        )
        .await
        {
            Ok(docs) => {
                let docs = docs.unwrap();
                let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
                return Json(contents).into_response();
            }
            Err(_) => {
                if retried {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                } else {
                    shared_database.write().await.reconfigure().await;
                    retried = true;
                    continue;
                }
            }
        }
    }

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
