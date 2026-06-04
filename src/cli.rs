use std::fmt;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub config_path: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    MissingConfigPath,
    MissingInputFiles,
    UnknownFlag(String),
}

impl Args {
    pub fn parse(raw_args: impl IntoIterator<Item = String>) -> Result<Self, CliError> {
        let mut config_path = None;
        let mut paths = Vec::new();
        let mut args = raw_args.into_iter();

        while let Some(arg) = args.next() {
            if arg == "-c" {
                let Some(path) = args.next() else {
                    return Err(CliError::MissingConfigPath);
                };
                config_path = Some(PathBuf::from(path));
            } else if arg.starts_with('-') {
                return Err(CliError::UnknownFlag(arg));
            } else {
                paths.push(PathBuf::from(arg));
            }
        }

        if paths.is_empty() {
            Err(CliError::MissingInputFiles)
        } else {
            Ok(Self { config_path, paths })
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingConfigPath => f.write_str("-c requires a config path"),
            CliError::MissingInputFiles => f.write_str("missing input file"),
            CliError::UnknownFlag(flag) => write!(f, "unknown flag: {flag}"),
        }
    }
}

pub fn usage(program: &str) -> String {
    format!("usage: {program} [-c config.toml] <file.tpl> [file.tpl ...]")
}

#[cfg(test)]
mod tests {
    use super::{Args, CliError};
    use std::path::PathBuf;

    #[test]
    fn parses_files_without_config() {
        let args = Args::parse(["one.tpl".to_string(), "two.tpl".to_string()]).expect("args");

        assert_eq!(
            args,
            Args {
                config_path: None,
                paths: vec![PathBuf::from("one.tpl"), PathBuf::from("two.tpl")]
            }
        );
    }

    #[test]
    fn parses_config_before_files() {
        let args = Args::parse([
            "-c".to_string(),
            "smarty-checker.toml".to_string(),
            "one.tpl".to_string(),
        ])
        .expect("args");

        assert_eq!(
            args,
            Args {
                config_path: Some(PathBuf::from("smarty-checker.toml")),
                paths: vec![PathBuf::from("one.tpl")]
            }
        );
    }

    #[test]
    fn rejects_missing_config_path() {
        let error = Args::parse(["-c".to_string()]).expect_err("missing config path");

        assert_eq!(error, CliError::MissingConfigPath);
    }

    #[test]
    fn rejects_unknown_flags() {
        let error = Args::parse(["--config".to_string()]).expect_err("unknown flag");

        assert_eq!(error, CliError::UnknownFlag("--config".to_string()));
    }
}
