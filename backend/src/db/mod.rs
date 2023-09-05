use cozo::*;
pub mod q;

pub fn start_db(db: DbInstance) -> DbInstance {
    let db = &db;
    {
        // Users:
        // Create missing tables: users
        // Create missing user: admin

        if q::ensure_users_table(db).is_err() {
            let result = q::create_users_table(db);
            assert!(result.is_ok());
        }

        struct UserData {
            username: String,
            default_pwd: String,
        }

        let admin_user_data = UserData {
            username: "admin".into(),
            default_pwd: "def".into(),
        };

        // if admin has not been created - create one with default password
        if q::find_user_by_username(db, &admin_user_data.username)
            .unwrap()
            .is_none()
        {
            q::put_user(
                db,
                &admin_user_data.username,
                &auth::hash_pwd(admin_user_data.default_pwd.as_bytes()).unwrap(),
            )
            .unwrap();
        }
    }

    {
        // Articles:
        // Create missing tables: articles
        if q::ensure_articles_table(db).is_err() {
            let result = q::create_articles_table(db);
            assert!(result.is_ok());
        }
    }

    {
        // Sessions:
        // Create missing tables: sessions
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

#[cfg(test)]
mod tests {
    use super::q;
    use claim::{assert_err, assert_none, assert_ok};
    use interfacing::trait_imports::*;

    #[allow(unused)]
    fn db() -> cozo::DbInstance {
        // in memory db
        cozo::DbInstance::default()
    }

    #[test]
    fn users_test() {
        let db = &db();

        assert_err!(q::ensure_users_table(db));
        assert_ok!(q::create_users_table(db));
        assert_ok!(q::ensure_users_table(db));

        struct UserData {
            username: String,
            pwd_hash: String,
        }

        let user_data = UserData {
            username: "admin".into(),
            pwd_hash: auth::hash_pwd(String::default().as_bytes()).unwrap(),
        };

        assert_ok!(q::put_user(db, &user_data.username, &user_data.pwd_hash));

        let user = q::find_user_by_username(db, &user_data.username)
            .expect("op to succeed")
            .expect("to find the user");

        assert_eq!(&user.username, &user_data.username);
        assert_eq!(&user.pwd_hash, &user_data.pwd_hash);

        let user_data = UserData {
            pwd_hash: auth::hash_pwd("updated-pwd".as_bytes()).unwrap(),
            ..user_data
        };

        assert_ok!(q::update_user_pwd_hash(
            db,
            &user_data.username,
            &user_data.pwd_hash
        ));
        let user = q::find_user_by_username(db, &user_data.username)
            .expect("op to succeed")
            .expect("to find the user");

        assert_eq!(&user.username, &user_data.username);
        assert_eq!(&user.pwd_hash, &user_data.pwd_hash);
    }

    #[test]
    fn articles_test() {
        let db = &db();

        assert_err!(q::ensure_articles_table(db));
        assert_ok!(q::create_articles_table(db));
        assert_ok!(q::ensure_articles_table(db));

        let assert_article_count =
            |count: usize| assert_eq!(q::find_articles(db).expect("op to succeed").len(), count);

        assert_article_count(0);

        let article_data = interfacing::Article::default();
        assert_ok!(q::put_article(db, article_data.clone()));
        assert_article_count(1);

        let article = q::find_article_by_public_id(db, &article_data.body().public_id)
            .expect("op to succeed")
            .expect("to find the article");
        assert_eq!(&article.body().public_id, &article_data.public_id);
        assert_eq!(&article.body().title, &article_data.title);
        assert_eq!(&article.body().markdown, &article_data.markdown);
        assert_eq!(&article.body().draft, &article_data.draft);

        let updated_article_data = interfacing::ArticleWithId {
            id: article.id,
            body: interfacing::Article {
                public_id: "updated".into(),
                markdown: "updated".into(),
                title: "updated".into(),
                draft: false,
            },
        };

        assert_ok!(q::update_article(db, updated_article_data.clone()));
        assert_article_count(1);
        let article = q::find_article_by_public_id(db, &updated_article_data.body().public_id)
            .expect("op to succeed")
            .expect("to find the article");

        assert_eq!(&article, &updated_article_data);

        assert_ok!(q::rm_article(db, &updated_article_data.id));
        assert_article_count(0);
    }

    #[test]
    fn sessions_test() {
        let db = &db();

        assert_err!(q::ensure_sessions_table(db));
        assert_ok!(q::create_sessions_table(db));
        assert_ok!(q::ensure_sessions_table(db));

        #[derive(Default)]
        struct SessionData {
            id: String,
            value: String,
        }

        let session_data = SessionData::default();

        assert_ok!(q::put_session(db, &session_data.id, &session_data.value));

        let session = q::find_session_by_id(db, &session_data.id)
            .expect("op to succeed")
            .expect("to find the session");
        assert_eq!(&session, &session_data.value);

        assert_ok!(q::rm_session(db, &session_data.id));
        let session = q::find_session_by_id(db, &session_data.id).expect("op to succeed");
        assert_none!(session);
    }
}
