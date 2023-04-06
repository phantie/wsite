use crate::routes::imports::*;

pub async fn logout(mut session: WritableSession) -> Response {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => {}
        Some(_user_id) => {
            session.destroy();
            tracing::info!("User successfully logged out.");
        }
    }

    routes().root.login.redirect_to().into_response()
}
