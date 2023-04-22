use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn new_article(
    State(state): State<AppState>,
    _session: ReadableSession,
    Json(body): Json<BodyData>,
) -> Response {
    // let _user_id: u64 = reject_anonymous_users(&session).unwrap();

    let articles = &state.database.collections.articles;

    Article {
        title: body.title,
        public_id: body.public_id,
        markdown: body.markdown,
    }
    .push_into_async(articles)
    .await
    .unwrap();

    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    public_id: String,
    markdown: String,
}

#[axum_macros::debug_handler]
pub async fn update_article(
    State(state): State<AppState>,
    _session: ReadableSession,
    Json(body): Json<BodyData>,
) -> Response {
    let articles = &state.database.collections.articles;

    let mapped_articles = articles
        .view::<ArticleByPublicID>()
        .with_key(body.public_id)
        .query_with_collection_docs()
        .await
        .unwrap();

    match mapped_articles.into_iter().next() {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(mapped_doc) => {
            let mut doc = mapped_doc.document.clone();
            doc.contents.title = body.title;
            doc.contents.markdown = body.markdown;
            doc.update_async(articles).await.unwrap();
            StatusCode::OK.into_response()
        }
    }
}
