use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Article {
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct ArticleWithId {
    pub id: String,
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

impl From<ArticleWithId> for Article {
    fn from(
        ArticleWithId {
            id: _,
            public_id,
            title,
            markdown,
            draft,
        }: ArticleWithId,
    ) -> Self {
        Self {
            public_id,
            title,
            markdown,
            draft,
        }
    }
}
