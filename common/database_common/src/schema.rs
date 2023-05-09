use bonsaidb::core::{
    document::{BorrowedDocument, Emit},
    schema::{
        Collection, ReduceResult, SerializedCollection, View, ViewMapResult, ViewMappedValue,
        ViewSchema,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Collection)]
#[collection(name = "shapes")]
pub struct Shape {
    pub sides: u32,
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "users",  views = [UserByUsername])]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, View)]
#[view(collection = User, key = String, value = u32, name = "by-username")]
pub struct UserByUsername;

impl ViewSchema for UserByUsername {
    type View = Self;

    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let user = User::document_contents(document)?;
        document.header.emit_key_and_value(user.username, 1)
    }

    fn version(&self) -> u64 {
        2
    }

    fn unique(&self) -> bool {
        true
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<Self::View>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        Ok(mappings.iter().map(|mapping| mapping.value).sum())
    }
}
