use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DatabaseInfo {
    pub is_running: bool,
}
