use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn new_article(
    State(state): State<AppState>,
    _session: ReadableSession,
    Json(body): Json<interfacing::Article>,
) -> Response {
    // let _user_id: u64 = reject_anonymous_users(&session).unwrap();

    let valid_public_id_charset = body
        .public_id
        .chars()
        .all(|c| char::is_alphanumeric(c) || ['-'].contains(&c));

    let invalid_public_id = body.public_id.is_empty() || !valid_public_id_charset;
    let invalid_title = body.title.is_empty();

    if invalid_public_id || invalid_title {
        return StatusCode::BAD_REQUEST.into_response();
    }

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

#[axum_macros::debug_handler]
pub async fn update_article(
    State(state): State<AppState>,
    _session: ReadableSession,
    Json(body): Json<interfacing::Article>,
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

#[axum_macros::debug_handler]
pub async fn delete_article(
    State(state): State<AppState>,
    _session: ReadableSession,
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
        Some(mapped_doc) => {
            mapped_doc.document.delete_async(articles).await.unwrap();
            StatusCode::OK.into_response()
        }
    }
}
