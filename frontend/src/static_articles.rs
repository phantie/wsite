pub fn static_articles() -> StaticArticles {
    StaticArticles {
        md_article_editor: StaticArticle {
            title: "Markdown article editor".into(),
            public_id: "md-article-editor".into(),
        },
        about: StaticArticle {
            title: "About".into(),
            public_id: "about".into(),
        },
        snake: StaticArticle {
            title: "Snake".into(),
            public_id: "snake".into(),
        },
    }
}

pub struct StaticArticles {
    pub md_article_editor: StaticArticle,
    pub about: StaticArticle,
    pub snake: StaticArticle,
}

pub struct StaticArticle {
    pub title: String,
    pub public_id: String,
}

impl IntoIterator for StaticArticles {
    type Item = StaticArticle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.about, self.md_article_editor, self.snake].into_iter()
    }
}
