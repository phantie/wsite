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

pub trait ArticleBody {
    fn body(&self) -> &Article;

    fn body_mut(&mut self) -> &mut Article;
}

impl ArticleBody for Article {
    fn body(&self) -> &Article {
        self
    }

    fn body_mut(&mut self) -> &mut Article {
        self
    }
}

impl ArticleBody for ArticleWithId {
    fn body(&self) -> &Article {
        &self.body
    }

    fn body_mut(&mut self) -> &mut Article {
        &mut self.body
    }
}
