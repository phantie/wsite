use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PasswordChangeForm {
    #[serde(serialize_with = "expose_secret_string")]
    pub current_password: SecretString,
    #[serde(serialize_with = "expose_secret_string")]
    pub new_password: SecretString,
    #[serde(serialize_with = "expose_secret_string")]
    pub new_password_check: SecretString,
}
