use serde::{Deserialize, Deserializer};

use crate::common::Error;
use std::fmt::Display;

use serde::de::Error as DeserializeError;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    #[serde()]
    pub environment: Environment,
    pub database: DatabaseConfiguration,
    pub server: ServerConfiguration,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfiguration {
    pub url: String,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfiguration {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub scheme: String,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Environment {
    Production,
    Development,
}

impl Environment {
    pub fn development() -> Self {
        Environment::Development
    }

    pub fn production() -> Self {
        Environment::Production
    }

    pub fn is_dev(&self) -> bool {
        match self {
            Self::Development => true,
            Self::Production => false,
        }
    }
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Environment::try_from(s.as_ref())
            .map_err(|_| DeserializeError::custom("Error parsing environment value"))
    }
}

impl TryFrom<&str> for Environment {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "production" | "prod" => Ok(Environment::Production),
            "development" | "dev" => Ok(Environment::Development),
            _ => Err(format!("Invalid environment value: {}", value).into()),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Environment::try_from(value.as_str())
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Environment::Production => "production",
            Environment::Development => "development",
        })
    }
}

/// Multipurpose function that helps detect the current environment the application
/// is running using the `MANIFOLD__ENVIRONMENT` environment variable.
///
/// \`\`\`
/// MANIFOLD__ENVIRONMENT = development | production.
/// \`\`\`
///
/// After detection, it loads the appropriate .yaml file
/// then it loads the environment variable that overrides whatever is set in the .yaml file.
/// For this to work, the environment variable MUST be in uppercase and starts with `MANIFOLD`,
/// a `__` separator then the category of config,
/// followed by `_` separator,  and then the variable, e.g.
/// `MANIFOLD__APPLICATION_PORT=5001` for `port` to be set as `5001`
pub fn get_config() -> Result<Configuration, Error> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let settings_directory = base_path.join("settings");

    // Detect the running environment.
    // Default to `development` if unspecified.
    let environment: Environment = std::env::var("MANIFOLD__ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse MANIFOLD__ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment);
    let config = config::Config::builder()
        .add_source(config::File::from(settings_directory.join("base.yaml")))
        .add_source(config::File::from(
            settings_directory.join(environment_filename),
        ))
        // Add in config from environment variables (with a prefix of MANIFOLD and '_' as separator)
        // E.g. `MANIFOLD__SERVER_PORT=5001 would set `Config.server.port` to 5001.
        .add_source(
            config::Environment::with_prefix("MANIFOLD")
                .prefix_separator("__")
                .separator("_"),
        )
        .build()?;

    config
        .try_deserialize::<Configuration>()
        .map_err(|err| err.into())
}
