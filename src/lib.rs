pub mod diagnostic;
pub mod lint;
pub mod parser;

pub use diagnostic::{Diagnostic, Position, Range, Severity};
pub use lint::{LintContext, Rule, default_rules, lint_source};
