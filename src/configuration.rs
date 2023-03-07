#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub dir: String,
}

pub fn get_configuration() -> Settings {
    let config = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build();

    config.unwrap().try_deserialize().unwrap()
}
