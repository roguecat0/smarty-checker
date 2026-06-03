use std::env;
use std::fs;
use std::process::ExitCode;

use smarty_checker::{Diagnostic, lint_source};

fn main() -> ExitCode {
    let paths: Vec<String> = env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: smarty-checker <file.tpl> [file.tpl ...]");
        return ExitCode::from(2);
    }

    let mut found_diagnostics = false;
    let mut found_operational_error = false;

    for path in paths {
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{path}: failed to read file: {error}");
                found_operational_error = true;
                continue;
            }
        };

        match lint_source(&source) {
            Ok(diagnostics) => {
                if !diagnostics.is_empty() {
                    found_diagnostics = true;
                    for diagnostic in diagnostics {
                        print_diagnostic(&path, &diagnostic);
                    }
                }
            }
            Err(error) => {
                eprintln!("{path}: {error}");
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

fn print_diagnostic(path: &str, diagnostic: &Diagnostic) {
    println!(
        "{}:{}:{}: {}[{}] {}",
        path,
        diagnostic.range.start.row + 1,
        diagnostic.range.start.column + 1,
        diagnostic.severity,
        diagnostic.rule_id,
        diagnostic.message
    );
}
