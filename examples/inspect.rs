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
            name: "void_tags",
            source: r#"{if $x}<div><br><img src="{$src}"><input disabled></div>{/if}"#,
        },
        Case {
            name: "optional_html_end_tags",
            source: "{if $x}<ul><li>One<li>Two</ul>{/if}",
        },
        Case {
            name: "smarty_controls_inside_tag_list",
            source: r#"{if $x}<select>{foreach $items as $item}<option>{$item}</option>{/foreach}</select>{/if}"#,
        },
        Case {
            name: "dynamic_tag_name",
            source: "{if $x}<{$tag}>content</{$tag}>{/if}",
        },
        Case {
            name: "self_closing_custom_element",
            source: r#"{if $x}<x-widget /><div></div>{/if}"#,
        },
        Case {
            name: "conditional_attribute",
            source: r#"{if $x}<button {if $disabled}disabled{/if}>Save</button>{/if}"#,
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
        let html_tree = parse_html(case.source);

        println!("\nsmarty sexp:\n{}", smarty_tree.root_node().to_sexp());
        println!("\nsmarty named nodes:");
        print_named_nodes(case.source, smarty_tree.root_node(), 0);

        println!("\nhtml sexp:\n{}", html_tree.root_node().to_sexp());
        println!("\nhtml named nodes:");
        print_named_nodes(case.source, html_tree.root_node(), 0);
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

fn print_named_nodes(source: &str, node: Node, depth: usize) {
    if node.is_named() {
        let indent = "  ".repeat(depth);
        let text = node
            .utf8_text(source.as_bytes())
            .unwrap_or("")
            .replace('\n', "\\n");
        println!(
            "{}{} {:?}-{:?} bytes {}..{} {:?}",
            indent,
            node.kind(),
            node.start_position(),
            node.end_position(),
            node.start_byte(),
            node.end_byte(),
            truncate(&text, 80)
        );
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        print_named_nodes(source, child, depth + 1);
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
