use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn all_shapes(
    Extension(db_client): Extension<SharedDbClient>,
) -> ApiResult<Json<Vec<schema::Shape>>> {
    tracing::info!("{:?}", db_client.read().await);
    HangingStrategy::default()
        .execute(
            |db_client| async {
                async move {
                    let shapes = &db_client.read().await.collections().shapes;
                    let docs = schema::Shape::all_async(shapes).await?;
                    Ok(Json(collect_contents(docs)))
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}

#[axum_macros::debug_handler]
pub async fn new_shape(
    Extension(db_client): Extension<SharedDbClient>,
    Json(body): Json<schema::Shape>,
) -> ApiResult<()> {
    HangingStrategy::default()
        .execute(
            |db_client| async {
                let body = body.clone();
                async move {
                    let shapes = &db_client.read().await.collections().shapes;
                    body.push_into_async(shapes).await.map_err(|e| e.error)?;
                    Ok(())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}
