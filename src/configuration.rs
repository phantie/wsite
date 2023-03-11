use serde::Deserialize;
// to deserialize variables set via env vars
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub testing: Testing,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub dir: String,
}

#[derive(Deserialize)]
pub struct Testing {
    pub database: DatabaseSettings,
}

pub fn get_configuration() -> Settings {
    fn conf_path(conf_dir: &std::path::PathBuf, filename: &str) -> String {
        conf_dir
            .join(filename)
            .into_os_string()
            .into_string()
            .unwrap()
    }

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let config = config::Config::builder()
        .add_source(
            config::File::with_name(&conf_path(&configuration_directory, "base")).required(true),
        )
        .add_source(
            config::File::with_name(&conf_path(&configuration_directory, environment.as_str()))
                .required(true),
        )
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build();

    config.unwrap().try_deserialize().unwrap()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
