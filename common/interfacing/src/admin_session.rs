use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AdminSession {
    pub user_id: u64,
    pub username: String,
}
