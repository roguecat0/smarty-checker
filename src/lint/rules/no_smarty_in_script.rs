use tree_sitter::Node;

use crate::diagnostic::{Diagnostic, Severity};
use crate::lint::html::{ScriptBodyRange, script_body_ranges};
use crate::lint::smarty::{is_control_flow, is_smarty_construct};
use crate::lint::{LintContext, Rule};
use crate::parser::range_from_node;

const LENIENT_RULE_ID: &str = "smarty/no-control-flow-in-script";
const STRICT_RULE_ID: &str = "smarty/no-smarty-in-script";

pub struct NoControlFlowInScript;
pub struct NoSmartyInScript;

enum Mode {
    Lenient,
    Strict,
}

impl Rule for NoControlFlowInScript {
    fn id(&self) -> &'static str {
        LENIENT_RULE_ID
    }

    fn check(&self, ctx: &LintContext<'_>, diagnostics: &mut Vec<Diagnostic>) {
        check_script_bodies(ctx, Mode::Lenient, diagnostics);
    }
}

impl Rule for NoSmartyInScript {
    fn id(&self) -> &'static str {
        STRICT_RULE_ID
    }

    fn check(&self, ctx: &LintContext<'_>, diagnostics: &mut Vec<Diagnostic>) {
        check_script_bodies(ctx, Mode::Strict, diagnostics);
    }
}

fn check_script_bodies(ctx: &LintContext<'_>, mode: Mode, diagnostics: &mut Vec<Diagnostic>) {
    let script_bodies = script_body_ranges(ctx.source);
    if script_bodies.is_empty() {
        return;
    }

    check_node(
        ctx.smarty_tree.root_node(),
        &script_bodies,
        &mode,
        diagnostics,
    );
}

fn check_node(
    node: Node,
    script_bodies: &[ScriptBodyRange],
    mode: &Mode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if matches_mode(node, mode) && is_inside_script_body(node, script_bodies) {
        diagnostics.push(Diagnostic {
            rule_id: rule_id(mode),
            message: message(node, mode),
            severity: Severity::Error,
            range: range_from_node(node),
        });

        // For a matched Smarty construct, reporting nested Smarty nodes creates duplicate noise.
        return;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        check_node(child, script_bodies, mode, diagnostics);
    }
}

fn matches_mode(node: Node, mode: &Mode) -> bool {
    match mode {
        Mode::Lenient => is_control_flow(node),
        Mode::Strict => is_smarty_construct(node),
    }
}

fn is_inside_script_body(node: Node, script_bodies: &[ScriptBodyRange]) -> bool {
    script_bodies.iter().any(|script_body| {
        node.start_byte() >= script_body.start_byte && node.end_byte() <= script_body.end_byte
    })
}

fn rule_id(mode: &Mode) -> &'static str {
    match mode {
        Mode::Lenient => LENIENT_RULE_ID,
        Mode::Strict => STRICT_RULE_ID,
    }
}

fn message(node: Node, mode: &Mode) -> String {
    match mode {
        Mode::Lenient => format!(
            "Smarty control-flow statement `{}` is not allowed inside a script body",
            node.kind()
        ),
        Mode::Strict => format!(
            "Smarty statement `{}` is not allowed inside a script body",
            node.kind()
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::lint::rules::{NoSmartyInScript, run_rule_for_test};
    use crate::lint::{LintContext, lint_source};

    fn default_messages(source: &str) -> Vec<String> {
        lint_source(source)
            .expect("lint should run")
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_id == "smarty/no-control-flow-in-script")
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn default_rule_rejects_if_inside_script_body() {
        let diagnostics = default_messages("<script>{if $x}init();{/if}</script>");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("if"));
    }

    #[test]
    fn default_rule_rejects_foreach_inside_script_body() {
        let diagnostics =
            default_messages("<script>{foreach $items as $item}init();{/foreach}</script>");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("foreach"));
    }

    #[test]
    fn default_rule_allows_inline_output_inside_script_body_for_now() {
        let diagnostics = default_messages("<script>const id = {$id};</script>");

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn default_rule_allows_smarty_in_script_tag_attributes() {
        let diagnostics = default_messages(r#"<script src="{$asset_url}"></script>"#);

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn default_rule_allows_control_flow_wrapping_script_element() {
        let diagnostics = default_messages("{if $x}<script>init();</script>{/if}");

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn strict_rule_rejects_inline_output_inside_script_body() {
        let ctx = LintContext::new("<script>const id = {$id};</script>").expect("context");
        let diagnostics = run_rule_for_test(&NoSmartyInScript, &ctx);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "smarty/no-smarty-in-script");
        assert!(diagnostics[0].message.contains("inline"));
    }

    #[test]
    fn strict_rule_still_allows_smarty_in_script_tag_attributes() {
        let ctx = LintContext::new(r#"<script src="{$asset_url}"></script>"#).expect("context");
        let diagnostics = run_rule_for_test(&NoSmartyInScript, &ctx);

        assert_eq!(diagnostics, Vec::new());
    }
}
