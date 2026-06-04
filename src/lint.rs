pub mod html;
pub mod rules;
pub mod smarty;
pub mod source;

use tree_sitter::Tree;

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::parser::{ParseSetupError, parse_smarty};

pub use rules::{RuleConfigError, default_rules, validate_config};

pub struct LintContext<'a> {
    pub source: &'a str,
    pub smarty_tree: Tree,
}

pub trait Rule {
    fn id(&self) -> &'static str;
    fn check(&self, ctx: &LintContext<'_>, diagnostics: &mut Vec<Diagnostic>);
}

impl<'a> LintContext<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseSetupError> {
        Ok(Self {
            source,
            smarty_tree: parse_smarty(source)?,
        })
    }
}

pub fn lint_source(source: &str) -> Result<Vec<Diagnostic>, ParseSetupError> {
    lint_source_with_config(source, &Config::default()).map_err(|error| match error {
        LintError::Parse(error) => error,
        LintError::RuleConfig(error) => unreachable!("{error}"),
    })
}

pub fn lint_source_with_config(
    source: &str,
    config: &Config,
) -> Result<Vec<Diagnostic>, LintError> {
    let ctx = LintContext::new(source)?;
    let mut diagnostics = Vec::new();

    for rule in rules::configured_rules(config)? {
        rule.check(&ctx, &mut diagnostics);
    }

    Ok(diagnostics)
}

#[derive(Debug)]
pub enum LintError {
    Parse(ParseSetupError),
    RuleConfig(RuleConfigError),
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintError::Parse(error) => error.fmt(f),
            LintError::RuleConfig(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LintError {}

impl From<ParseSetupError> for LintError {
    fn from(error: ParseSetupError) -> Self {
        LintError::Parse(error)
    }
}

impl From<RuleConfigError> for LintError {
    fn from(error: RuleConfigError) -> Self {
        LintError::RuleConfig(error)
    }
}
