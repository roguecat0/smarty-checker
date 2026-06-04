use tree_sitter::Node;

pub fn is_control_flow(node: Node) -> bool {
    is_control_flow_kind(node.kind())
}

pub fn is_control_flow_kind(kind: &str) -> bool {
    matches!(
        kind,
        "if_block"
            | "elseif_block"
            | "else_block"
            | "foreach_block"
            | "foreachelse_block"
            | "for_block"
            | "forelse_block"
            | "section_block"
            | "sectionelse_block"
            | "while_block"
            | "block"
    )
}

pub fn is_control_flow_parent(node: Node) -> bool {
    node.parent()
        .map(|parent| is_control_flow_kind(parent.kind()))
        .unwrap_or(false)
}

pub fn is_smarty_construct(node: Node) -> bool {
    matches!(
        node.kind(),
        "tag"
            | "block"
            | "if_block"
            | "elseif_block"
            | "else_block"
            | "foreach_block"
            | "foreachelse_block"
            | "for_block"
            | "forelse_block"
            | "section_block"
            | "sectionelse_block"
            | "while_block"
            | "literal_block"
    )
}

pub fn direct_html_fragment(source: &str, body: Node) -> String {
    let body_source = &source[body.start_byte()..body.end_byte()];
    let mut bytes = body_source.as_bytes().to_vec();

    let mut cursor = body.walk();
    for child in body.named_children(&mut cursor) {
        if is_control_flow(child) {
            let start = child.start_byte() - body.start_byte();
            let end = child.end_byte() - body.start_byte();
            for byte in &mut bytes[start..end] {
                *byte = b' ';
            }
        }
    }

    String::from_utf8(bytes).expect("source came from a UTF-8 Rust string")
}
