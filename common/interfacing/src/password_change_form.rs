use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug)]
pub struct PasswordChangeForm {
    pub current_password: SecretString,
    pub new_password: SecretString,
    pub new_password_check: SecretString,
}

impl Serialize for PasswordChangeForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PasswordChangeForm", 3)?;
        s.serialize_field("current_password", &self.current_password.expose_secret())?;
        s.serialize_field("new_password", &self.new_password.expose_secret())?;
        s.serialize_field(
            "new_password_check",
            &self.new_password_check.expose_secret(),
        )?;
        s.end()
    }
}
