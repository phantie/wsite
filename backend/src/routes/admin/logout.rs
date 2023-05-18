use crate::routes::imports::*;

pub async fn logout(mut session: WritableSession) {
    match session.get::<u64>("user_id") {
        None => {}
        Some(user_id) => {
            session.destroy();
            tracing::info!("User {user_id} successfully logged out.");
        }
    }
}
