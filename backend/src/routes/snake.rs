use crate::routes::imports::*;

use crate::startup::mp_snake;

#[axum_macros::debug_handler]
pub async fn create_lobby(
    Extension(lobbies): Extension<mp_snake::Lobbies>,
    lobby: Json<interfacing::snake::CreateLobby>,
) -> ApiResult<impl IntoResponse> {
    tracing::info!("{:?}", lobby);

    let lobby = mp_snake::Lobby::new(lobby.name.clone());
    let result = lobbies.insert_if_missing(lobby).await;

    match result {
        Ok(()) => Ok(StatusCode::OK),
        Err(msg) => Err(ApiError::Conflict(msg)),
    }
}
