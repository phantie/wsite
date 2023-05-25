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

/// Verifies whether [password_candidate] transofrmed to hash
/// with the scheme of [password_hash_challenger] equals to [password_hash_challenger] hash
///
/// Does not take into account argon2::Argon2 params
pub fn verify_password_hash(
    password_hash_challenger: impl AsRef<str>,
    password_candidate: &[u8],
) -> argon2::password_hash::Result<()> {
    let password_hash_challenger = argon2::PasswordHash::new(password_hash_challenger.as_ref())?;

    argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        password_candidate,
        &password_hash_challenger,
    )
}

pub fn invalid_password_hash() -> String {
    "$argon2id$v=19$m=15000,t=2,p=1$\
    gZiV/M1gPc22ElAH/Jh1Hw$\
    CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
        .into()
}
