pub fn static_articles() -> StaticArticles {
    StaticArticles {
        md_article_editor: StaticArticle {
            title: "Markdown article editor".into(),
            public_id: "md-article-editor".into(),
        },
    }
}

pub struct StaticArticle {
    pub title: String,
    pub public_id: String,
}

pub struct StaticArticles {
    pub md_article_editor: StaticArticle,
}

impl IntoIterator for StaticArticles {
    type Item = StaticArticle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.md_article_editor].into_iter()
    }
}
