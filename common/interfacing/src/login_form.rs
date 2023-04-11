use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: SecretString,
}

impl Serialize for LoginForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("LoginForm", 2)?;
        s.serialize_field("username", &self.username)?;
        s.serialize_field("password", &self.password.expose_secret())?;
        s.end()
    }
}
