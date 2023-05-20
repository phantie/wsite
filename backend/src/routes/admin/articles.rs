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

fn reject_invalid_article(article: &interfacing::Article) -> ApiResult<()> {
    if valid_article(article) {
        Ok(())
    } else {
        Err(ApiError::BadRequest)
    }
}

#[axum_macros::debug_handler]
pub async fn new_article(
    session: ReadableSession,
    Extension(db_client): Extension<SharedDbClient>,
    Json(body): Json<interfacing::Article>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;
    reject_invalid_article(&body)?;

    HangingStrategy::default()
        .execute(
            |db_client| async {
                let body = body.clone();
                async move {
                    let articles = &db_client.read().await.collections().articles;
                    schema::Article {
                        title: body.title,
                        public_id: body.public_id,
                        markdown: body.markdown,
                        draft: body.draft,
                    }
                    .push_into_async(articles)
                    .await
                    .map_err(|e| e.error)?;
                    Ok(())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}

#[axum_macros::debug_handler]
pub async fn update_article(
    session: ReadableSession,
    Extension(db_client): Extension<SharedDbClient>,
    Json(body): Json<interfacing::Article>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;
    reject_invalid_article(&body)?;

    HangingStrategy::default()
        .execute(
            |db_client| async {
                let body = body.clone();
                async move {
                    let articles = &db_client.read().await.collections().articles;
                    let docs = articles
                        .view::<schema::ArticleByPublicID>()
                        .with_key(&body.public_id)
                        .query_with_collection_docs()
                        .await?;

                    let doc = docs.into_iter().next().ok_or(ApiError::EntryNotFound)?;

                    let mut doc = doc.document.clone();
                    doc.contents.title = body.title;
                    doc.contents.markdown = body.markdown;
                    doc.contents.draft = body.draft;
                    doc.update_async(articles).await?;
                    Ok(())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}

#[axum_macros::debug_handler]
pub async fn delete_article(
    session: ReadableSession,
    Path(public_id): Path<String>,
    Extension(db_client): Extension<SharedDbClient>,
) -> ApiResult<impl IntoResponse> {
    reject_anonymous_users(&session)?;

    HangingStrategy::default()
        .execute(
            |db_client| async {
                let public_id = public_id.clone();
                async move {
                    let articles = &db_client.read().await.collections().articles;
                    let docs = articles
                        .view::<schema::ArticleByPublicID>()
                        .with_key(&public_id)
                        .query_with_collection_docs()
                        .await?;

                    let doc = docs.into_iter().next().ok_or(ApiError::EntryNotFound)?;

                    doc.document.delete_async(articles).await?;
                    Ok(())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}

pub async fn article_list(
    session: ReadableSession,
    Extension(db_client): Extension<SharedDbClient>,
) -> ApiResult<Json<Vec<schema::Article>>> {
    let docs = HangingStrategy::default()
        .execute(
            |db_client| async {
                async move {
                    let articles = &db_client.read().await.collections().articles;
                    let docs = schema::Article::all_async(articles).await?;

                    ApiResult::<_>::Ok(docs)
                }
                .await
            },
            db_client.clone(),
        )
        .await??;

    let contents = collect_contents(docs);

    let contents = match reject_anonymous_users(&session) {
        Ok(_) => contents,
        Err(_) => contents.into_iter().filter(|a| !a.draft).collect(),
    };

    Ok(Json(contents))
}

pub async fn article_by_public_id(
    Path(public_id): Path<String>,
    Extension(db_client): Extension<SharedDbClient>,
) -> ApiResult<impl IntoResponse> {
    HangingStrategy::default()
        .execute(
            |db_client| async {
                let public_id = public_id.clone();
                async move {
                    let articles = &db_client.read().await.collections().articles;

                    let docs = articles
                        .view::<schema::ArticleByPublicID>()
                        .with_key(&public_id)
                        .query_with_collection_docs()
                        .await?;

                    let doc = docs.into_iter().next().ok_or(ApiError::EntryNotFound)?;
                    Ok(Json(&doc.document.contents).into_response())
                }
                .await
            },
            db_client.clone(),
        )
        .await?
}

// pub fn one_doc<S, T, I>(docs: I) -> ApiResult<T>
// where
//     I: IntoIterator<Item = T>,
// {
//     let mut docs = docs.into_iter();
//     let doc = docs.next().ok_or(ApiError::EntryNotFound)?;

//     if let Some(_) = docs.next() {
//         Err(ApiError::UnexpectedError(anyhow::anyhow!(
//             "maximum one document must be returned"
//         )))?
//     }

//     Ok(doc)
// }
