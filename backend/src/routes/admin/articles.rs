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
    session: ReadableSession,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(body): Json<interfacing::Article>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    if !valid_article(&body) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let articles = &shared_database.read().await.articles().await?;
    schema::Article {
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
    session: ReadableSession,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
    Json(body): Json<interfacing::Article>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    if !valid_article(&body) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let articles = &shared_database.read().await.articles().await?;

    let mapped_articles = articles
        .view::<schema::ArticleByPublicID>()
        .with_key(&body.public_id)
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
    session: ReadableSession,
    Path(public_id): Path<String>,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> Result<impl IntoResponse, ApiError> {
    reject_anonymous_users(&session)?;

    let articles = &shared_database.read().await.articles().await?;

    let mapped_articles = articles
        .view::<schema::ArticleByPublicID>()
        .with_key(&public_id)
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
    session: ReadableSession,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> Result<Json<Vec<schema::Article>>, ApiError> {
    let articles = &shared_database.read().await.articles().await?;

    let docs = schema::Article::all_async(articles).await.unwrap();

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

    Ok(Json(contents))
}

pub async fn article_by_public_id(
    Path(public_id): Path<String>,
    Extension(shared_database): Extension<SharedRemoteDatabase>,
) -> Result<Response, ApiError> {
    let articles = &shared_database.read().await.articles().await?;

    let mapped_articles = articles
        .view::<schema::ArticleByPublicID>()
        .with_key(&public_id)
        .query_with_collection_docs()
        .await
        .unwrap();

    match mapped_articles.into_iter().next() {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(article) => Ok(Json(&article.document.contents).into_response()),
    }
}
