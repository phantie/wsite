use crate::routes::imports::*;
use remote_database::shema::Shape;

#[axum_macros::debug_handler]
#[allow(unused_mut)]
pub async fn all_shapes(State(mut state): State<AppState>) -> Response {
    // // Original solution to hanging client replaced by perpetural database pinging
    // let shapes = state.remote_database.collections.shapes.clone();
    // let mut retried = false;
    // loop {
    //     match tokio::time::timeout_at(
    //         tokio::time::Instant::now() + std::time::Duration::from_secs(5),
    //         Shape::all_async(&shapes),
    //     )
    //     .await
    //     {
    //         Ok(docs) => {
    //             let docs = docs.unwrap();
    //             let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
    //             return Json(contents).into_response();
    //         }
    //         Err(_) => {
    //             if retried {
    //                 return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    //             } else {
    //                 state.remote_database.reconfigure().await;
    //                 retried = true;
    //                 continue;
    //             }
    //         }
    //     }
    // }

    let shapes = state.remote_database.collections.shapes;
    let docs = Shape::all_async(&shapes).await.unwrap();
    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
    Json(contents).into_response()
}

#[axum_macros::debug_handler]
pub async fn new_shape(State(state): State<AppState>, Json(body): Json<Shape>) -> Response {
    let shapes = state.remote_database.collections.shapes;
    body.push_into_async(&shapes).await.unwrap();
    StatusCode::OK.into_response()
}
