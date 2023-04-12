pub use secrecy::{ExposeSecret, SecretString};
pub use serde::{Deserialize, Serialize};

pub fn expose_secret_string<S>(v: &SecretString, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(v.expose_secret())
}
