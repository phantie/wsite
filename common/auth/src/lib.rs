fn hash_algo<'a>() -> argon2::Argon2<'a> {
    argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(15000, 2, 1, None).unwrap(),
    )
}

pub fn hash_pwd<'a>(password: impl Into<&'a [u8]>) -> anyhow::Result<String> {
    Ok(argon2::PasswordHasher::hash_password(
        &hash_algo(),
        password.into(),
        &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()),
    )?
    .to_string())
}
