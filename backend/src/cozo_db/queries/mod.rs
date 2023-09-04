#![allow(unused)]
use std::collections::BTreeMap;

use cozo::*;
use itertools::Itertools;

pub mod utils;
use utils::Error;
use utils::*;

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
