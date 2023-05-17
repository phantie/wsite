use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use clap::Parser;
use database_common::schema;
use rpassword::read_password;
use std::io::Write;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let configuration = get_configuration();

    let storage = Arc::new(
        storage(
            &configuration.database.dir,
            configuration.database.memory_only,
        )
        .await,
    );

    let args = Cli::parse();
    let username = args.username;

    let database = Database::init(storage).await;

    let users = &database.collections.users;

    let user_count = users
        .view::<schema::UserByUsername>()
        .with_key(&username)
        .reduce()
        .await
        .unwrap();

    if user_count > 0 {
        panic!("user already exists");
    }

    print!("Type a password: ");
    std::io::stdout().flush().unwrap();
    let password = read_password().unwrap();
    // println!("The password is: '{}'", password);

    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string();

    schema::User {
        username: username.clone(),
        password_hash: password_hash,
    }
    .push_into_async(&database.collections.users)
    .await
    .unwrap();

    // let user_docs = User::all_async(&database.collections.users).await.unwrap();

    // dbg!(user_docs);
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(clap::Parser, Debug)]
struct Cli {
    /// Username
    #[arg(short, long)]
    username: String,
}
