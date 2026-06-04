pub mod cli;
pub mod config;
pub mod diagnostic;
pub mod lint;
pub mod parser;

pub use config::Config;
pub use diagnostic::{Diagnostic, Position, Range, Severity};
pub use lint::{
    LintContext, LintError, Rule, default_rules, lint_source, lint_source_with_config,
    validate_config,
};
