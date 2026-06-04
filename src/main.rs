use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use smarty_checker::cli::{Args, usage};
use smarty_checker::{Config, Diagnostic, lint_source_with_config, validate_config};

fn main() -> ExitCode {
    let program = env::args()
        .next()
        .unwrap_or_else(|| "smarty-checker".to_string());
    let args = match Args::parse(env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage(&program));
            return ExitCode::from(2);
        }
    };

    let config = match args.config_path {
        Some(path) => match Config::from_path(&path) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(2);
            }
        },
        None => Config::default(),
    };

    if let Err(error) = validate_config(&config) {
        eprintln!("{error}");
        return ExitCode::from(2);
    }

    let mut found_diagnostics = false;
    let mut found_operational_error = false;

    for path in args.paths {
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{}: failed to read file: {error}", path.display());
                found_operational_error = true;
                continue;
            }
        };

        match lint_source_with_config(&source, &config) {
            Ok(diagnostics) => {
                if !diagnostics.is_empty() {
                    found_diagnostics = true;
                    for diagnostic in diagnostics {
                        print_diagnostic(&path, &diagnostic);
                    }
                }
            }
            Err(error) => {
                eprintln!("{}: {error}", path.display());
                found_operational_error = true;
            }
        }
    }

    if found_operational_error {
        ExitCode::from(2)
    } else if found_diagnostics {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_diagnostic(path: &Path, diagnostic: &Diagnostic) {
    println!(
        "{}:{}:{}: {}[{}] {}",
        path.display(),
        diagnostic.range.start.row + 1,
        diagnostic.range.start.column + 1,
        diagnostic.severity,
        diagnostic.rule_id,
        diagnostic.message
    );
}
