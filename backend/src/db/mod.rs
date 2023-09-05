use cozo::*;
pub mod q;

pub fn start_db() -> DbInstance {
    let db = &cozo::DbInstance::default();
    // let db = &DbInstance::new("sqlite", "testing.db", Default::default()).unwrap();

    {
        // Users
        if q::ensure_users_table(db).is_err() {
            let result = q::create_users_table(db);
            assert!(result.is_ok());
        }

        let pwd = "a";
        let pwd_hash = auth::hash_pwd(pwd.as_bytes()).unwrap();

        q::put_user(db, "admin", &pwd_hash).unwrap();

        let user = q::find_user_by_username(db, "admin");
        assert_eq!(user.unwrap().unwrap().username, "admin");

        q::update_user_pwd_hash(db, "admin", &pwd_hash).unwrap();
        dbg!(q::find_user_by_username(db, "admin").unwrap());
    }

    {
        // Articles
        if q::ensure_articles_table(db).is_err() {
            let result = q::create_articles_table(db);
            assert!(result.is_ok());
        }

        let article = interfacing::Article::default();
        q::put_article(db, article.clone()).unwrap();

        let article = q::find_article_by_public_id(db, "").unwrap().unwrap();
        assert_eq!(article.public_id, "");

        let article_id = article.id;
        let mut article = interfacing::ArticleWithId::default();
        article.id = article_id;
        article.public_id = "updated".into();
        q::update_article(db, article).unwrap();
        let article = q::find_article_by_public_id(db, "updated").unwrap();
        assert!(article.is_some());
        assert_eq!(article.unwrap().public_id, "updated");
    }

    {
        // Sessions
        if q::ensure_sessions_table(db).is_err() {
            let result = q::create_sessions_table(db);
            assert!(result.is_ok());
        }
    }

    db.clone()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // most likely query syntax error
    #[error("Engine error")]
    EngineError(miette::ErrReport),
    // returned results don't cover expected cases
    #[error("Result error")]
    ResultError(NamedRows),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type OpResult = Result<()>;
