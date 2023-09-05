use crate::{configuration::get_env, db, routes::imports::*};
use interfacing::AdminSession;

#[axum_macros::debug_handler]
pub async fn admin_session(
    Extension(db): Extension<cozo::DbInstance>,
    session: ReadableSession,
) -> ApiResult<Json<AdminSession>> {
    // returns user info if logged in, else 403

    if get_env().local() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let session = match session.get::<String>("username") {
        None => Err(ApiError::AuthError(anyhow::anyhow!("Session missing")))?,
        Some(username) => {
            let user = db::q::find_user_by_username(&db, &username)?.unwrap(); // TODO safen
            let username = user.username;

            AdminSession { username }
        }
    };

    Ok(Json::from(session))
}
