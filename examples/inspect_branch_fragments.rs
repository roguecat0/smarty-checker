use tree_sitter::{Node, Parser, Tree};

struct Case {
    name: &'static str,
    source: &'static str,
}

fn main() {
    let cases = [
        Case {
            name: "nested_balanced_direct_html",
            source: "{if $x}<ul>{if $y}<li></li><li></li>{/if}</ul>{/if}",
        },
        Case {
            name: "cross_branch_balanced_full_file_only",
            source: "{if $x}<div>{else}</div>{/if}",
        },
        Case {
            name: "nested_inner_closes_outer",
            source: "{if $x}<ul>{if $y}</ul>{/if}{/if}",
        },
        Case {
            name: "smarty_in_attribute_and_text",
            source: r#"{if $x}<div class="{$class}">{$content}</div>{/if}"#,
        },
        Case {
            name: "smarty_controls_inside_tag_list",
            source: r#"{if $x}<select>{foreach $items as $item}<option>{$item}</option>{/foreach}</select>{/if}"#,
        },
        Case {
            name: "mismatched_direct_html",
            source: "{if $x}<section><div></section>{/if}",
        },
        Case {
            name: "unclosed_direct_html",
            source: "{if $x}<section><div></div>{/if}",
        },
        Case {
            name: "stray_direct_close",
            source: "{if $x}</section>{/if}",
        },
        Case {
            name: "void_and_self_closing",
            source: r#"{if $x}<div><br><img src="{$src}"><x-widget /></div>{/if}"#,
        },
        Case {
            name: "optional_html_end_tags",
            source: "{if $x}<ul><li>One<li>Two</ul>{/if}",
        },
        Case {
            name: "conditional_attribute",
            source: r#"{if $x}<button {if $disabled}disabled{/if}>Save</button>{/if}"#,
        },
        Case {
            name: "dynamic_tag_name",
            source: "{if $x}<{$tag}>content</{$tag}>{/if}",
        },
        Case {
            name: "script_raw_text_with_smarty",
            source: r#"{if $x}<script>if (x < y) { alert("{$msg}"); }</script>{/if}"#,
        },
        Case {
            name: "less_than_text",
            source: "{if $x}Price < 10<div></div>{/if}",
        },
    ];

    for case in cases {
        println!("\n=== {} ===", case.name);
        println!("source: {}", case.source);

        let smarty_tree = parse_smarty(case.source);
        inspect_branch_bodies(case.source, smarty_tree.root_node());
    }
}

fn parse_smarty(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_smarty2::LANGUAGE.into())
        .expect("smarty grammar should load");
    parser
        .parse(source, None)
        .expect("smarty parse should return a tree")
}

fn parse_html(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_html::LANGUAGE.into())
        .expect("html grammar should load");
    parser
        .parse(source, None)
        .expect("html parse should return a tree")
}

fn inspect_branch_bodies(source: &str, node: Node) {
    if node.kind() == "body" && is_control_flow_parent(node) {
        let fragment = direct_html_fragment(source, node);
        let html_tree = parse_html(&fragment);
        println!(
            "\n{} body bytes {}..{} {:?}",
            node.parent()
                .map(|parent| parent.kind())
                .unwrap_or("<root>"),
            node.start_byte(),
            node.end_byte(),
            fragment
        );
        println!("html: {}", html_tree.root_node().to_sexp());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        inspect_branch_bodies(source, child);
    }
}

fn is_control_flow_parent(node: Node) -> bool {
    node.parent()
        .map(|parent| {
            matches!(
                parent.kind(),
                "if" | "else_if" | "else" | "foreach" | "foreach_else" | "block" | "nocache"
            )
        })
        .unwrap_or(false)
}

fn direct_html_fragment(source: &str, body: Node) -> String {
    let body_source = &source[body.start_byte()..body.end_byte()];
    let mut bytes = body_source.as_bytes().to_vec();

    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        if is_nested_control_flow(child) {
            let start = child.start_byte() - body.start_byte();
            let end = child.end_byte() - body.start_byte();
            for byte in &mut bytes[start..end] {
                *byte = b' ';
            }
        }
    }

    String::from_utf8(bytes).expect("fixtures are utf-8")
}

fn is_nested_control_flow(node: Node) -> bool {
    matches!(
        node.kind(),
        "if" | "else_if" | "else" | "foreach" | "foreach_else" | "block" | "nocache"
    )
}
