use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AdminSession {
    pub username: String,
}
