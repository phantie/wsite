use crate::routes::imports::*;
use remote_database::shema::Shape;

#[axum_macros::debug_handler]
pub async fn all_shapes(State(state): State<AppState>) -> Json<Vec<Shape>> {
    let shapes = state.remote_database.collections.shapes;
    let docs = Shape::all_async(&shapes).await.unwrap();
    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();
    Json(contents)
}

#[axum_macros::debug_handler]
pub async fn new_shape(State(state): State<AppState>, Json(body): Json<Shape>) -> Response {
    let shapes = state.remote_database.collections.shapes;
    body.push_into_async(&shapes).await.unwrap();
    StatusCode::OK.into_response()
}
