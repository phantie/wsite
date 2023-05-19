use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn all_shapes(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> ApiResult<Json<Vec<schema::Shape>>> {
    tracing::info!("Remote database ID: {}", shared_database.read().await.id);
    HangingStrategy::default()
        .execute(
            |shared_database| async {
                async move {
                    let shapes = &shared_database.read().await.collections.shapes;
                    let docs = schema::Shape::all_async(shapes).await?;
                    Ok(Json(collect_contents(docs)))
                }
                .await
            },
            shared_database.clone(),
        )
        .await?
}

#[axum_macros::debug_handler]
pub async fn new_shape(
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(body): Json<schema::Shape>,
) -> ApiResult<()> {
    HangingStrategy::default()
        .execute(
            |shared_database| async {
                let body = body.clone();
                async move {
                    let shapes = shared_database.read().await.collections.shapes.clone();
                    body.push_into_async(&shapes).await.map_err(|e| e.error)?;
                    Ok(())
                }
                .await
            },
            shared_database.clone(),
        )
        .await?
}
