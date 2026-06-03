pub mod html;
pub mod rules;
pub mod smarty;
pub mod source;

use tree_sitter::Tree;

use crate::diagnostic::Diagnostic;
use crate::parser::{ParseSetupError, parse_smarty};

pub use rules::default_rules;

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
    let ctx = LintContext::new(source)?;
    let mut diagnostics = Vec::new();

    for rule in default_rules() {
        rule.check(&ctx, &mut diagnostics);
    }

    Ok(diagnostics)
}
