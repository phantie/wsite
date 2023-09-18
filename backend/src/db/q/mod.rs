mod utils;

use imports::*;

mod imports {
    pub use super::utils::{Error, *};
    pub use cozo::*;
    pub use itertools::Itertools;
    pub use std::collections::BTreeMap;
}

#[tracing::instrument(name = "Create users table", skip_all)]
pub fn create_users_table(db: &DbInstance) -> OpResult {
    let script = include_str!("users/create_users_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Ensure users table", skip_all)]
pub fn ensure_users_table(db: &DbInstance) -> OpResult {
    let script = include_str!("users/ensure_users_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Find user by username", skip_all)]
pub fn find_user_by_username(db: &DbInstance, username: &str) -> Result<Option<User>> {
    let script = include_str!("users/find_by_username.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "username".into() => username.into()
    };
    let result = db
        .run_script(script, params, ScriptMutability::Mutable)
        .map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match (&headers[..], &rows[..]) {
        (["username", "pwd_hash"], [[DataValue::Str(username), DataValue::Str(pwd_hash)]]) => {
            Ok(Some(User {
                username: username.to_string(),
                pwd_hash: pwd_hash.to_string(),
            }))
        }
        (["username", "pwd_hash"], []) => Ok(None),
        _ => Err(Error::ResultError(result)),
    }
}

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub pwd_hash: String,
}

#[tracing::instrument(name = "Put user", skip_all)]
pub fn put_user(db: &DbInstance, username: &str, pwd_hash: &str) -> OpResult {
    let script = include_str!("users/put.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "username".into() => username.into(),
        "pwd_hash".into() => pwd_hash.into()
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Update user pwd_hash", skip_all)]
pub fn update_user_pwd_hash(db: &DbInstance, username: &str, pwd_hash: &str) -> OpResult {
    let script = include_str!("users/update_pwd_hash.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "username".into() => username.into(),
        "pwd_hash".into() => pwd_hash.into()
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Create articles table", skip_all)]
pub fn create_articles_table(db: &DbInstance) -> OpResult {
    let script = include_str!("articles/create_articles_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Ensure articles table", skip_all)]
pub fn ensure_articles_table(db: &DbInstance) -> OpResult {
    let script = include_str!("articles/ensure_articles_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[derive(Default, Clone)]
pub struct Article {
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

#[derive(Debug, Default)]
pub struct ArticleWithId {
    pub id: String,
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

#[tracing::instrument(name = "Put article", skip_all)]
pub fn put_article(db: &DbInstance, article: interfacing::Article) -> OpResult {
    let script = include_str!("articles/put.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "title".into() => article.title.into(),
        "public_id".into() => article.public_id.into(),
        "markdown".into() => article.markdown.into(),
        "draft".into() => article.draft.into(),
    };

    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Find article by public_id", skip_all)]
pub fn find_article_by_public_id(
    db: &DbInstance,
    public_id: &str,
) -> Result<Option<interfacing::ArticleWithId>> {
    let script = include_str!("articles/find_by_public_id.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "public_id".into() => public_id.into()
    };
    let result = db
        .run_script(script, params, ScriptMutability::Mutable)
        .map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match (&headers[..], &rows[..]) {
        (
            ["id", "public_id", "title", "markdown", "draft"],
            [[DataValue::Uuid(UuidWrapper(id)), DataValue::Str(public_id), DataValue::Str(title), DataValue::Str(markdown), DataValue::Bool(draft)]],
        ) => Ok(Some(interfacing::ArticleWithId {
            id: id.to_string(),
            body: interfacing::Article {
                title: title.to_string(),
                public_id: public_id.to_string(),
                markdown: markdown.to_string(),
                draft: *draft,
            },
        })),
        (["id", "public_id", "title", "markdown", "draft"], []) => Ok(None),
        _ => Err(Error::ResultError(result)),
    }
}

#[tracing::instrument(name = "Update article", skip_all)]
pub fn update_article(db: &DbInstance, article: interfacing::ArticleWithId) -> OpResult {
    let script = include_str!("articles/update.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "id".into() => DataValue::Uuid(UuidWrapper(uuid::Uuid::parse_str(&article.id).unwrap())), // TODO safen
        "title".into() => article.body().title.clone().into(),
        "public_id".into() => article.body().public_id.clone().into(),
        "markdown".into() => article.body().markdown.clone().into(),
        "draft".into() => article.body().draft.into(),
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Find articles", skip_all)]
pub fn find_articles(db: &DbInstance) -> Result<Vec<interfacing::ArticleWithId>> {
    let script = include_str!("articles/find.cozo");
    let result = db
        .run_script(script, Default::default(), ScriptMutability::Mutable)
        .map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match &headers[..] {
        ["id", "public_id", "title", "markdown", "draft"] => {}
        _ => return Err(Error::ResultError(result)),
    }

    let mut res = vec![];
    // all rows must comply to format, if any does not - return error
    for row in rows {
        match &row[..] {
            [DataValue::Uuid(UuidWrapper(id)), DataValue::Str(public_id), DataValue::Str(title), DataValue::Str(markdown), DataValue::Bool(draft)] =>
            {
                res.push(interfacing::ArticleWithId {
                    id: id.to_string(),
                    body: interfacing::Article {
                        title: title.to_string(),
                        public_id: public_id.to_string(),
                        markdown: markdown.to_string(),
                        draft: *draft,
                    },
                });
            }
            _ => return Err(Error::ResultError(result)),
        }
    }

    Ok(res)
}

#[tracing::instrument(name = "Remove article", skip(db))]
pub fn rm_article(db: &DbInstance, id: &str) -> OpResult {
    let script = include_str!("articles/rm.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "id".into() => id.into(),
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Create sessions table", skip_all)]
pub fn create_sessions_table(db: &DbInstance) -> OpResult {
    let script = include_str!("sessions/create_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Ensure sessions table", skip_all)]
pub fn ensure_sessions_table(db: &DbInstance) -> OpResult {
    let script = include_str!("sessions/ensure_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Put session", ret, skip(db))]
pub fn put_session(db: &DbInstance, id: &str, value: &str) -> OpResult {
    let script = include_str!("sessions/put.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "id".into() => id.into(),
        "value".into() => value.into()
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Find session by id", ret, skip(db))]
pub fn find_session_by_id<'de>(db: &DbInstance, id: &str) -> Result<Option<String>> {
    let script = include_str!("sessions/find_by_id.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "id".into() => id.into()
    };
    let result = db
        .run_script(script, params, ScriptMutability::Mutable)
        .map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match (&headers[..], &rows[..]) {
        (["id", "value"], [[DataValue::Str(_id), DataValue::Str(value)]]) => {
            Ok(Some(value.to_string()))
        }
        (["id", "value"], []) => Ok(None),
        _ => Err(Error::ResultError(result)),
    }
}

#[tracing::instrument(name = "Remove session", skip(db))]
pub fn rm_session(db: &DbInstance, id: &str) -> OpResult {
    let script = include_str!("sessions/rm.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "id".into() => id.into(),
    };
    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Create endpoint_hits table", skip_all)]
pub fn create_endpoint_hits(db: &DbInstance) -> OpResult {
    let script = include_str!("endpoint_hits/create_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Ensure endpoint_hits table", skip_all)]
pub fn ensure_endpoint_hits(db: &DbInstance) -> OpResult {
    let script = include_str!("endpoint_hits/ensure_table.cozo");
    let result = db.run_script(script, Default::default(), ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Put endpoint hit", skip_all)]
pub fn put_endpoint_hit(db: &DbInstance, value: interfacing::EndpointHit) -> OpResult {
    let script = include_str!("endpoint_hits/put.cozo");
    let params: BTreeMap<String, DataValue> = map_macro::btree_map! {
        "hashed_ip".into() => value.hashed_ip.into(),
        "endpoint".into() => value.endpoint.into(),
        "method".into() => value.method.into(),
        "status".into() => (value.status as i64).into(),
        "timestamp".into() => value.timestamp.into(),
    };

    let result = db.run_script(script, params, ScriptMutability::Mutable);
    op_result(result)
}

#[tracing::instrument(name = "Find endpoint_hits", skip_all)]
pub fn find_endpoint_hits(db: &DbInstance) -> Result<Vec<interfacing::EndpointHit>> {
    let script = include_str!("endpoint_hits/find.cozo");
    let result = db
        .run_script(script, Default::default(), ScriptMutability::Mutable)
        .map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match &headers[..] {
        ["hashed_ip", "endpoint", "method", "status", "timestamp"] => {}
        _ => return Err(Error::ResultError(result)),
    }

    let mut res = vec![];
    // all rows must comply to format, if any does not - return error
    for row in rows {
        match &row[..] {
            [DataValue::Str(hashed_ip), DataValue::Str(endpoint), DataValue::Str(method), DataValue::Num(Num::Int(status)), DataValue::Str(timestamp)] =>
            {
                res.push(interfacing::EndpointHit {
                    hashed_ip: hashed_ip.to_string(),
                    endpoint: endpoint.to_string(),
                    method: method.to_string(),
                    status: *status as u16,
                    timestamp: timestamp.to_string(),
                });
            }
            _ => return Err(Error::ResultError(result)),
        }
    }

    Ok(res)
}
