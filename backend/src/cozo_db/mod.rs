use cozo::*;
pub mod queries;

pub fn start_db() -> DbInstance {
    let db = &cozo::DbInstance::default();
    // let db = &DbInstance::new("sqlite", "testing.db", Default::default()).unwrap();

    if queries::ensure_users_table(db).is_err() {
        let result = queries::create_users_table(db);
        assert!(result.is_ok());
    }

    if queries::ensure_articles_table(db).is_err() {
        let result = queries::create_articles_table(db);
        assert!(result.is_ok());
    }

    let pwd_hash = auth::hash_pwd("a".as_bytes()).unwrap();

    queries::put_user(db, "admin", &pwd_hash).unwrap();

    dbg!(queries::find_user_by_username(db, "admin").unwrap());

    queries::update_user_pwd_hash(db, "admin", &pwd_hash).unwrap();

    dbg!(queries::find_user_by_username(db, "admin").unwrap());

    db.clone()
}
