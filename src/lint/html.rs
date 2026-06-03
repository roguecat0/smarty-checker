use crate::diagnostic::Range;
use crate::lint::source::range_for_offsets;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagEvent {
    Open { name: String, range: Range },
    Close { name: String, range: Range },
    SelfClosing { name: String, range: Range },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptBodyRange {
    pub start_byte: usize,
    pub end_byte: usize,
}

pub fn tag_events_from_fragment(
    source: &str,
    base_byte: usize,
    base_position: tree_sitter::Point,
) -> Vec<TagEvent> {
    let bytes = source.as_bytes();
    let mut events = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'<' {
            index += 1;
            continue;
        }

        if source[index..].starts_with("<!--") {
            index = source[index..]
                .find("-->")
                .map(|end| index + end + 3)
                .unwrap_or(bytes.len());
            continue;
        }

        if source[index..].starts_with("</") {
            let Some(tag_end) = find_tag_end(source, index + 1) else {
                break;
            };
            if let Some((name, _name_start, _name_end)) = read_tag_name(source, index + 2) {
                events.push(TagEvent::Close {
                    name,
                    range: range_for_offsets(source, base_byte, base_position, index, tag_end + 1),
                });
            }
            index = tag_end + 1;
        } else if matches!(bytes.get(index + 1), Some(b'!' | b'?')) {
            let Some(tag_end) = find_tag_end(source, index + 1) else {
                break;
            };
            index = tag_end + 1;
        } else if let Some((name, _name_start, _name_end)) = read_tag_name(source, index + 1) {
            let Some(tag_end) = find_tag_end(source, index + 1) else {
                break;
            };
            let range = range_for_offsets(source, base_byte, base_position, index, tag_end + 1);
            if is_self_closing_source(&source[index..tag_end]) {
                events.push(TagEvent::SelfClosing { name, range });
            } else {
                events.push(TagEvent::Open { name, range });
            }
            index = tag_end + 1;
        } else {
            index += 1;
        }
    }

    events
}

pub fn script_body_ranges(source: &str) -> Vec<ScriptBodyRange> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let Some(open_start_relative) = find_case_insensitive(&source[index..], "<script") else {
            break;
        };
        let open_start = index + open_start_relative;

        if !is_tag_name_boundary(bytes.get(open_start + "<script".len()).copied()) {
            index = open_start + 1;
            continue;
        }

        let Some(open_end) = find_tag_end(source, open_start + 1) else {
            break;
        };

        let body_start = open_end + 1;
        let Some(close_start_relative) = find_case_insensitive(&source[body_start..], "</script")
        else {
            break;
        };
        let close_start = body_start + close_start_relative;

        ranges.push(ScriptBodyRange {
            start_byte: body_start,
            end_byte: close_start,
        });

        index = close_start + "</script".len();
    }

    ranges
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let needle = needle.as_bytes();
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn is_tag_name_boundary(byte: Option<u8>) -> bool {
    !matches!(
        byte,
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b':' | b'_')
    )
}

fn find_tag_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut index = start;

    while index < bytes.len() {
        match (bytes[index], quote) {
            (b'"' | b'\'', None) => quote = Some(bytes[index]),
            (byte, Some(current_quote)) if byte == current_quote => quote = None,
            (b'>', None) => return Some(index),
            _ => {}
        }
        index += 1;
    }

    None
}

fn read_tag_name(source: &str, start: usize) -> Option<(String, usize, usize)> {
    let bytes = source.as_bytes();
    let mut index = start;
    while matches!(bytes.get(index), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        index += 1;
    }

    let name_start = index;
    if !matches!(bytes.get(index), Some(b'a'..=b'z' | b'A'..=b'Z')) {
        return None;
    }

    while matches!(
        bytes.get(index),
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b':' | b'_')
    ) {
        index += 1;
    }

    Some((
        source[name_start..index].to_ascii_lowercase(),
        name_start,
        index,
    ))
}

fn is_self_closing_source(tag_source: &str) -> bool {
    tag_source.trim_end().ends_with('/')
}

#[cfg(test)]
mod tests {
    use super::script_body_ranges;

    #[test]
    fn extracts_script_body_without_attributes() {
        let source = "<script>body</script>";
        let ranges = script_body_ranges(source);

        assert_eq!(ranges.len(), 1);
        assert_eq!(&source[ranges[0].start_byte..ranges[0].end_byte], "body");
    }

    #[test]
    fn extracts_script_body_with_attributes() {
        let source = r#"<script src="{$url}" type="module">body</script>"#;
        let ranges = script_body_ranges(source);

        assert_eq!(ranges.len(), 1);
        assert_eq!(&source[ranges[0].start_byte..ranges[0].end_byte], "body");
    }

    #[test]
    fn does_not_match_scriptish_tag_names() {
        assert!(script_body_ranges("<scriptlet>{if $x}</scriptlet>").is_empty());
    }
}
