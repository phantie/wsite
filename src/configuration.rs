#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {}

pub fn get_configuration() -> Result<config::Config, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
}
