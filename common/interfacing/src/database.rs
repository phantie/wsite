use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DatabaseInfo {
    pub is_running: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DatabaseRestart {
    pub backup_location: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DatabaseCreateBackup {
    pub backup_location: String,
}
