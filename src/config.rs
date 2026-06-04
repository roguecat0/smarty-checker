use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct Config {
    rule_overrides: HashMap<String, bool>,
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
        source: toml::de::Error,
    },
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    rules: HashMap<String, bool>,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let parsed: ConfigFile =
            toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                path: path.display().to_string(),
                source,
            })?;

        Ok(Self {
            rule_overrides: parsed.rules,
        })
    }

    pub fn is_rule_enabled(&self, rule_id: &str, default_enabled: bool) -> bool {
        self.rule_overrides
            .get(rule_id)
            .copied()
            .unwrap_or(default_enabled)
    }

    pub fn overrides(&self) -> &HashMap<String, bool> {
        &self.rule_overrides
    }

    #[cfg(test)]
    pub fn from_toml_for_test(contents: &str) -> Self {
        let parsed: ConfigFile = toml::from_str(contents).expect("test config should parse");

        Self {
            rule_overrides: parsed.rules,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, source } => {
                write!(f, "{path}: failed to read config: {source}")
            }
            ConfigError::Parse { path, source } => {
                write!(f, "{path}: failed to parse config: {source}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
