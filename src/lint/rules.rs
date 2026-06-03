mod balanced_html_in_control_flow;
mod no_smarty_in_script;

use crate::lint::Rule;

pub use balanced_html_in_control_flow::BalancedHtmlInControlFlow;
pub use no_smarty_in_script::{NoControlFlowInScript, NoSmartyInScript};

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(BalancedHtmlInControlFlow),
        Box::new(NoControlFlowInScript),
    ]
}

#[cfg(test)]
pub fn run_rule_for_test(
    rule: &dyn Rule,
    ctx: &crate::lint::LintContext<'_>,
) -> Vec<crate::diagnostic::Diagnostic> {
    let mut diagnostics = Vec::new();
    rule.check(ctx, &mut diagnostics);
    diagnostics
}
