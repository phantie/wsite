use cozo::*;
pub mod queries;

pub fn start_db() -> DbInstance {
    let db = &cozo::DbInstance::default();
    // let db = &DbInstance::new("sqlite", "testing.db", Default::default()).unwrap();

    {
        // Users
        if queries::ensure_users_table(db).is_err() {
            let result = queries::create_users_table(db);
            assert!(result.is_ok());
        }

        let pwd = "a";
        let pwd_hash = auth::hash_pwd(pwd.as_bytes()).unwrap();

        queries::put_user(db, "admin", &pwd_hash).unwrap();

        let user = queries::find_user_by_username(db, "admin");
        assert_eq!(user.unwrap().unwrap().username, "admin");

        queries::update_user_pwd_hash(db, "admin", &pwd_hash).unwrap();
        dbg!(queries::find_user_by_username(db, "admin").unwrap());
    }

    {
        // Articles
        if queries::ensure_articles_table(db).is_err() {
            let result = queries::create_articles_table(db);
            assert!(result.is_ok());
        }

        queries::put_article(db, queries::Article::default()).unwrap();
        let article = queries::find_article_by_public_id(db, "").unwrap().unwrap();
        assert_eq!(article.public_id, "");
    }

    db.clone()
}
