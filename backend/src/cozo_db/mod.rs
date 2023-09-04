use cozo::*;
mod queries;

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

    queries::put_user(db, "admin", "hA5h").unwrap();

    dbg!(queries::find_user_by_username(db, "admin").unwrap());

    db.clone()
}
