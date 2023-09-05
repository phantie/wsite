use crate::db;
use crate::routes::imports::*;

#[tracing::instrument(name = "Is valid article")]
fn valid_article(article: impl Into<interfacing::Article> + std::fmt::Debug) -> bool {
    let article = article.into();
    let valid_public_id_charset = article
        .public_id
        .chars()
        .all(|c| char::is_alphanumeric(c) || ['-'].contains(&c));

    let valid_public_id = !article.public_id.is_empty() && valid_public_id_charset;
    let valid_title = !article.title.is_empty();

    valid_public_id && valid_title
}

#[tracing::instrument(name = "Reject invalid article", skip_all)]
fn reject_invalid_article(
    article: impl Into<interfacing::Article> + std::fmt::Debug,
) -> ApiResult<()> {
    if valid_article(article) {
        Ok(())
    } else {
        Err(ApiError::BadRequest)
    }
}

#[axum_macros::debug_handler]
pub async fn new_article(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
    Json(article): Json<interfacing::Article>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;
    reject_invalid_article(article.clone())?;
    db::q::put_article(&db, article.clone())?;
    let article = db::q::find_article_by_public_id(&db, &article.public_id)?.unwrap();
    Ok(Json(article))
}

#[axum_macros::debug_handler]
pub async fn update_article(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
    Json(article): Json<interfacing::ArticleWithId>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;
    reject_invalid_article(article.clone())?;
    db::q::update_article(&db, article)?;
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn delete_article(
    session: ReadableSession,
    Path(id): Path<String>,
    Extension(db): Extension<cozo::DbInstance>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;
    db::q::rm_article(&db, &id)?;
    Ok(())
}

pub async fn article_list(
    session: ReadableSession,
    Extension(db): Extension<cozo::DbInstance>,
) -> ApiResult<Json<Vec<interfacing::ArticleWithId>>> {
    let articles = db::q::find_articles(&db)?;
    let contents = match reject_anonymous_users(&session) {
        Ok(_) => articles,
        // hide draft articles from unauthorized
        Err(_) => articles
            .into_iter()
            .filter(|article| !article.draft)
            .collect(),
    };
    Ok(Json(contents))
}

pub async fn article_by_public_id(
    Path(public_id): Path<String>,
    Extension(db): Extension<cozo::DbInstance>,
) -> ApiResult<impl IntoResponse> {
    let article =
        db::q::find_article_by_public_id(&db, &public_id)?.ok_or(ApiError::EntryNotFound)?;

    Ok(Json(article))
}
