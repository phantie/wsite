use crate::routes::imports::*;

pub async fn logout(mut session: WritableSession) {
    match session.get::<String>("username") {
        None => {}
        Some(username) => {
            session.destroy();
            tracing::info!("User {username} successfully logged out.");
        }
    }
}
