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

        let article = interfacing::Article::default();
        queries::put_article(db, article.clone()).unwrap();

        let article = queries::find_article_by_public_id(db, "").unwrap().unwrap();
        assert_eq!(article.public_id, "");

        let article_id = article.id;
        let mut article = interfacing::ArticleWithId::default();
        article.id = article_id;
        article.public_id = "updated".into();
        queries::update_article(db, article).unwrap();
        let article = queries::find_article_by_public_id(db, "updated").unwrap();
        assert!(article.is_some());
        assert_eq!(article.unwrap().public_id, "updated");
    }

    {
        // Sessions
        if queries::ensure_sessions_table(db).is_err() {
            let result = queries::create_sessions_table(db);
            assert!(result.is_ok());
        }
    }

    db.clone()
}
