use crate::routes::imports::*;

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
