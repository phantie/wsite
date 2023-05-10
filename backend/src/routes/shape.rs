use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn all_shapes(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> Result<Json<Vec<schema::Shape>>, ApiError> {
    tracing::info!("Remote database ID: {}", shared_database.read().await.id);
    let docs = HangingStrategy::default()
        .execute(
            |shared_database| async {
                async move {
                    let shapes = &shared_database.read().await.collections.shapes;
                    schema::Shape::all_async(shapes).await
                }
                .await
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
    Json(body): Json<schema::Shape>,
) -> Response {
    let shapes = shared_database.read().await.collections.shapes.clone();
    body.push_into_async(&shapes).await.unwrap();
    StatusCode::OK.into_response()
}
