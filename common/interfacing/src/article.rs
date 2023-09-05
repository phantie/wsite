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
    pub body: Article,
}

impl Article {
    pub fn body(&self) -> &Article {
        self
    }

    pub fn body_mut(&mut self) -> &mut Article {
        self
    }
}

impl ArticleWithId {
    pub fn body(&self) -> &Article {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Article {
        &mut self.body
    }
}
