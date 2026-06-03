use tree_sitter::Node;

use crate::diagnostic::{Diagnostic, Range, Severity};
use crate::lint::html::{TagEvent, tag_events_from_fragment};
use crate::lint::smarty::{direct_html_fragment, is_control_flow_parent};
use crate::lint::{LintContext, Rule};

const RULE_ID: &str = "smarty/balanced-html-in-control-flow";

pub struct BalancedHtmlInControlFlow;

#[derive(Debug, Clone)]
struct OpenTag {
    name: String,
    range: Range,
}

impl Rule for BalancedHtmlInControlFlow {
    fn id(&self) -> &'static str {
        RULE_ID
    }

    fn check(&self, ctx: &LintContext<'_>, diagnostics: &mut Vec<Diagnostic>) {
        check_node(ctx.source, ctx.smarty_tree.root_node(), diagnostics);
    }
}

fn check_node(source: &str, node: Node, diagnostics: &mut Vec<Diagnostic>) {
    if node.kind() == "body" && is_control_flow_parent(node) {
        check_body(source, node, diagnostics);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        check_node(source, child, diagnostics);
    }
}

fn check_body(source: &str, body: Node, diagnostics: &mut Vec<Diagnostic>) {
    let fragment = direct_html_fragment(source, body);
    let events = tag_events_from_fragment(&fragment, body.start_byte(), body.start_position());
    validate_events(events, diagnostics);
}

fn validate_events(events: Vec<TagEvent>, diagnostics: &mut Vec<Diagnostic>) {
    let mut stack: Vec<OpenTag> = Vec::new();

    for event in events {
        match event {
            TagEvent::Open { name, range } => {
                if !is_void_element(&name) {
                    stack.push(OpenTag { name, range });
                }
            }
            TagEvent::Close { name, range } => {
                if let Some(index) = stack.iter().rposition(|tag| tag.name == name) {
                    for unclosed in stack.drain(index + 1..) {
                        diagnostics.push(unclosed_tag_diagnostic(&unclosed));
                    }
                    stack.pop();
                } else {
                    diagnostics.push(Diagnostic {
                        rule_id: RULE_ID,
                        message: format!(
                            "Closing </{name}> has no matching opening tag in this Smarty branch"
                        ),
                        severity: Severity::Error,
                        range,
                    });
                }
            }
            TagEvent::SelfClosing { .. } => {}
        }
    }

    for unclosed in stack {
        diagnostics.push(unclosed_tag_diagnostic(&unclosed));
    }
}

fn unclosed_tag_diagnostic(tag: &OpenTag) -> Diagnostic {
    Diagnostic {
        rule_id: RULE_ID,
        message: format!("Opening <{}> is not closed in this Smarty branch", tag.name),
        severity: Severity::Error,
        range: tag.range,
    }
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use crate::lint::lint_source;

    fn messages(source: &str) -> Vec<String> {
        lint_source(source)
            .expect("lint should run")
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn accepts_balanced_nested_control_flow() {
        let diagnostics = messages("{if $x}<ul>{if $y}<li></li><li></li>{/if}</ul>{/if}");

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn rejects_tags_balanced_only_across_branches() {
        let diagnostics = messages("{if $x}<div>{else}</div>{/if}");

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].contains("Opening <div> is not closed"));
        assert!(diagnostics[1].contains("Closing </div> has no matching opening tag"));
    }

    #[test]
    fn checks_nested_bodies_independently() {
        let diagnostics = messages("{if $x}<ul>{if $y}</ul>{/if}{/if}");

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].contains("Opening <ul> is not closed"));
        assert!(diagnostics[1].contains("Closing </ul> has no matching opening tag"));
    }

    #[test]
    fn accepts_smarty_inline_expressions_in_html() {
        let diagnostics = messages(r#"{if $x}<div class="{$class}">{$content}</div>{/if}"#);

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn accepts_void_and_self_closing_tags() {
        let diagnostics = messages(r#"{if $x}<div><br><img src="{$src}"><x-widget /></div>{/if}"#);

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn rejects_optional_end_tags() {
        let diagnostics = messages("{if $x}<ul><li>One<li>Two</ul>{/if}");

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].contains("Opening <li> is not closed"));
        assert!(diagnostics[1].contains("Opening <li> is not closed"));
    }

    #[test]
    fn rejects_mismatched_direct_html() {
        let diagnostics = messages("{if $x}<section><div></section>{/if}");

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("Opening <div> is not closed"));
    }

    #[test]
    fn dynamic_tag_names_are_unsupported_by_this_rule() {
        let diagnostics = messages("{if $x}<{$tag}>content</{$tag}>{/if}");

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn ignores_non_tag_less_than_text() {
        let diagnostics = messages("{if $x}Price < 10<div></div>{/if}");

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    #[test]
    fn accepts_conditional_attributes_by_checking_outer_and_inner_bodies() {
        let diagnostics =
            messages(r#"{if $x}<button {if $disabled}disabled{/if}>Save</button>{/if}"#);

        assert_eq!(diagnostics, Vec::<String>::new());
    }
}
