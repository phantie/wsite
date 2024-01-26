use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn create_lobby(
    lobby: Json<interfacing::snake::CreateLobby>,
) -> ApiResult<impl IntoResponse> {
    tracing::info!("{:?}", lobby);
    // TODO implement
    Ok(StatusCode::OK)
}
