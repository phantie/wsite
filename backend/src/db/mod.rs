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

#[cfg(test)]
mod tests {
    use super::q;
    use claim::{assert_err, assert_ok};

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
}
