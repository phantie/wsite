use crate::routes::imports::*;

pub async fn logout(mut session: WritableSession) -> StatusCode {
    let user_id: Option<u64> = session.get("user_id");

    match user_id {
        None => {}
        Some(_user_id) => {
            session.destroy();
            tracing::info!("User successfully logged out.");
        }
    }

    StatusCode::OK
}
