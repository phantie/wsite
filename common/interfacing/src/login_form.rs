use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginForm {
    pub username: String,
    #[serde(serialize_with = "expose_secret_string")]
    pub password: SecretString,
}
