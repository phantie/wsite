use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Article {
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}
