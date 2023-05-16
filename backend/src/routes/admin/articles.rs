use crate::routes::imports::*;

fn valid_article(article: &interfacing::Article) -> bool {
    let valid_public_id_charset = article
        .public_id
        .chars()
        .all(|c| char::is_alphanumeric(c) || ['-'].contains(&c));

    let valid_public_id = !article.public_id.is_empty() && valid_public_id_charset;
    let valid_title = !article.title.is_empty();

    valid_public_id && valid_title
}

#[axum_macros::debug_handler]
pub async fn new_article(
    State(state): State<AppState>,
    session: ReadableSession,
    Json(body): Json<interfacing::Article>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    if !valid_article(&body) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let articles = &state.database.collections.articles;
    Article {
        title: body.title,
        public_id: body.public_id,
        markdown: body.markdown,
        draft: body.draft,
    }
    .push_into_async(articles)
    .await
    .unwrap();

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn update_article(
    State(state): State<AppState>,
    session: ReadableSession,
    Json(body): Json<interfacing::Article>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    if !valid_article(&body) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let articles = &state.database.collections.articles;

    let mapped_articles = articles
        .view::<ArticleByPublicID>()
        .with_key(body.public_id)
        .query_with_collection_docs()
        .await
        .unwrap();

    match mapped_articles.into_iter().next() {
        None => Ok(StatusCode::NOT_FOUND),
        Some(mapped_doc) => {
            let mut doc = mapped_doc.document.clone();
            doc.contents.title = body.title;
            doc.contents.markdown = body.markdown;
            doc.contents.draft = body.draft;
            doc.update_async(articles).await.unwrap();
            Ok(StatusCode::OK)
        }
    }
}

#[axum_macros::debug_handler]
pub async fn delete_article(
    State(state): State<AppState>,
    session: ReadableSession,
    Path(public_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    let articles = &state.database.collections.articles;

    let mapped_articles = articles
        .view::<ArticleByPublicID>()
        .with_key(public_id)
        .query_with_collection_docs()
        .await
        .unwrap();

    match mapped_articles.into_iter().next() {
        None => Ok(StatusCode::NOT_FOUND),
        Some(mapped_doc) => {
            mapped_doc.document.delete_async(articles).await.unwrap();
            Ok(StatusCode::OK)
        }
    }
}

pub async fn article_list(
    State(state): State<AppState>,
    session: ReadableSession,
) -> Json<Vec<Article>> {
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

    let contents = match reject_anonymous_users(&session) {
        Ok(_) => contents,
        Err(_) => contents.into_iter().filter(|a| !a.draft).collect(),
    };

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
