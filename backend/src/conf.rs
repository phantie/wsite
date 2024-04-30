// Configuration definitions, functions and tests
//

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string as de_num;
use std::sync::Arc;

static ENV_PREFIX: &str = "BE";

fn prefixed_env(suffix: &str) -> String {
    format!("{}__{}", ENV_PREFIX, suffix)
}

#[derive(Clone, derived_deref::Deref)]
pub struct Conf {
    #[target]
    pub env_conf: Arc<EnvConf>,
    pub env: Env,
}

impl Conf {
    pub fn new(env: Env, env_conf: EnvConf) -> Self {
        Self {
            env_conf: Arc::new(env_conf),
            env,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConf {
    pub session_secret: Option<String>,
    #[serde(deserialize_with = "de_num")]
    pub port: u16,
    pub host: String,
    pub db: DbConf,
    pub log: Log,

    pub features: EnvFeatures,
}

#[derive(Deserialize, Clone, Debug)]
pub enum DbStorageEngine {
    Memory,
    SQLite,
    Sled,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DbConf {
    pub storage_engine: DbStorageEngine,
    pub path: String,
}

impl DbConf {
    pub fn db_instance(&self) -> cozo::DbInstance {
        cozo::DbInstance::new(
            match self.storage_engine {
                DbStorageEngine::Memory => "mem",
                DbStorageEngine::SQLite => "sqlite",
                DbStorageEngine::Sled => "sled",
            },
            &self.path,
            Default::default(),
        )
        .unwrap()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct EnvFeatures {}

#[derive(Deserialize, Debug, Clone)]
pub struct Log {
    pub pretty: bool,
}

impl EnvConf {
    pub fn derive(env: Env) -> Self {
        fn join_filename(conf_dir: &std::path::PathBuf, filename: &str) -> String {
            conf_dir
                .join(filename)
                .into_os_string()
                .into_string()
                .unwrap()
        }

        let conf_dir = std::env::var(prefixed_env("CONF_DIR"))
            .map(|v| std::path::PathBuf::from(v))
            .unwrap_or_else(|_| {
                let base_path = std::env::current_dir().unwrap();
                base_path.join("conf")
            });

        let conf_builder = config::Config::builder()
            .add_source(
                config::File::with_name(&join_filename(&conf_dir, "default")).required(true),
            )
            .add_source(
                config::File::with_name(&join_filename(&conf_dir, env.as_ref())).required(false),
            )
            .add_source(config::Environment::with_prefix(ENV_PREFIX).separator("__"))
            .build();

        let conf = conf_builder.unwrap();

        match conf.try_deserialize() {
            Ok(conf) => conf,
            Err(e) => {
                dbg!(&e);
                Err(e).expect("correct config")
            }
        }
    }

    pub fn test_default() -> Self {
        // TODO just load local profile and override fields if needed
        Self {
            port: 0,
            session_secret: Some("d51563a0e65c0645e59cb5fe2fd1970cddae21cee6e916912ee4c766928a5032c582e433195809f6adc6b1ce9dd5d21136dc6fa51a4ac099ad883118c6185109".into()),
            host: "127.0.0.1".into(),
            db: DbConf {
                storage_engine: DbStorageEngine::Memory,
                path: String::new(),
            },
            features: EnvFeatures {},
            log: Log { pretty: false },
        }
    }
}

use derive_more::Display;

#[derive(Debug, PartialEq, Display, Clone, Copy)]
pub enum Env {
    #[display(fmt = "local")]
    Local,
    #[display(fmt = "prod")]
    Prod,
}

impl Env {
    pub fn derive() -> Self {
        // One variable to rule all
        let glob_env = std::env::var("SNK_ENV").unwrap_or_else(|_| "local".into());

        // Or set a more specific per executable
        std::env::var(prefixed_env("ENV"))
            .unwrap_or(glob_env)
            .try_into()
            .expect("valid variable")
    }

    pub fn local(&self) -> bool {
        matches!(self, Self::Local)
    }

    pub fn prod(&self) -> bool {
        matches!(self, Self::Prod)
    }
}

impl AsRef<str> for Env {
    fn as_ref(&self) -> &str {
        match self {
            Self::Local => "local",
            Self::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Env {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "prod" => Ok(Self::Prod),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `prod`.",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use envtestkit::{lock::lock_test, set_env};

    #[test]
    fn default_current_env() {
        assert!(Env::derive().local());
    }

    #[test]
    fn default_current_env_not() {
        assert!(!Env::derive().prod());
    }

    #[test]
    fn env() {
        #[derive(Debug)]
        struct Test<'a> {
            glob_env: Option<&'a str>,
            local_env: Option<&'a str>,
            result: Result<Env, ()>,
        }

        impl<'a> Test<'a> {
            fn run(&self) {
                let _lock = lock_test();

                let _1 = self.glob_env.map(|env| set_env("SNK_ENV".into(), env));

                let _2 = self
                    .local_env
                    .map(|env| set_env(prefixed_env("ENV").into(), env));

                match &self.result {
                    #[allow(unused)]
                    Ok(expected) => {
                        assert_eq!(&Env::derive(), expected, "{:?}", self);
                    }
                    Err(()) => {
                        let result = std::panic::catch_unwind(|| Env::derive());
                        assert!(result.is_err(), "{:?}", self);
                    }
                }
            }
        }

        // Successful cases
        {
            Test {
                glob_env: Some(Env::Prod.as_ref()),
                local_env: None,
                result: Ok(Env::Prod),
            }
            .run();

            Test {
                glob_env: Some(Env::Local.as_ref()),
                local_env: None,
                result: Ok(Env::Local),
            }
            .run();

            Test {
                glob_env: None,
                local_env: None,
                result: Ok(Env::Local),
            }
            .run();

            Test {
                glob_env: None,
                local_env: Some(Env::Local.as_ref()),
                result: Ok(Env::Local),
            }
            .run();

            Test {
                glob_env: None,
                local_env: Some(Env::Prod.as_ref()),
                result: Ok(Env::Prod),
            }
            .run();

            Test {
                glob_env: Some(Env::Local.as_ref()),
                local_env: Some(Env::Local.as_ref()),
                result: Ok(Env::Local),
            }
            .run();

            Test {
                glob_env: Some(Env::Local.as_ref()),
                local_env: Some(Env::Prod.as_ref()),
                result: Ok(Env::Prod),
            }
            .run();

            Test {
                glob_env: Some(Env::Prod.as_ref()),
                local_env: Some(Env::Local.as_ref()),
                result: Ok(Env::Local),
            }
            .run();

            Test {
                glob_env: Some(Env::Prod.as_ref()),
                local_env: Some(Env::Prod.as_ref()),
                result: Ok(Env::Prod),
            }
            .run();
        }

        // Unsuccessful cases
        {
            let invalid_env_value = "";

            Test {
                glob_env: Some(invalid_env_value),
                local_env: None,
                result: Err(()),
            }
            .run();

            Test {
                glob_env: Some(invalid_env_value),
                local_env: None,
                result: Err(()),
            }
            .run();

            Test {
                glob_env: Some(invalid_env_value),
                local_env: Some(invalid_env_value),
                result: Err(()),
            }
            .run();
        }
    }
}
