use bonsaidb::core::schema::Collection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Collection)]
#[collection(name = "shapes")]
pub struct Shape {
    pub sides: u32,
}
