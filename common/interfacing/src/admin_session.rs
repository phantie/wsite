use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdminSession {
    pub user_id: u64,
    pub username: String,
}
