use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub testing: Testing,
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
    let config = config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build();

    config.unwrap().try_deserialize().unwrap()
}
