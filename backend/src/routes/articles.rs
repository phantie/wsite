use crate::routes::imports::*;

pub async fn all_articles(State(state): State<AppState>) -> Json<Vec<Article>> {
    let docs = Article::all_async(&state.database.collections.articles)
        .await
        .unwrap();

    // for doc in docs {
    //     let _r = doc
    //         .delete_async(&state.database.collections.articles)
    //         .await
    //         .unwrap();
    // }
    // let contents = vec![];

    let contents = docs.into_iter().map(|doc| doc.contents).collect::<Vec<_>>();

    Json(contents)
}

pub async fn article_by_public_id(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
) -> Response {
    let articles = &state.database.collections.articles;

    let mapped_articles = articles
        .view::<ArticleByPublicID>()
        .with_key(public_id)
        .query_with_collection_docs()
        .await
        .unwrap();

    match mapped_articles.into_iter().next() {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(article) => Json(&article.document.contents).into_response(),
    }
}
