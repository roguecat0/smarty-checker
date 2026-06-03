use crate::diagnostic::{Position, Range};

pub fn range_for_offsets(
    source: &str,
    base_byte: usize,
    base_position: tree_sitter::Point,
    start: usize,
    end: usize,
) -> Range {
    Range {
        start: position_for_offset(source, base_position, start),
        end: position_for_offset(source, base_position, end),
        start_byte: base_byte + start,
        end_byte: base_byte + end,
    }
}

pub fn position_for_offset(source: &str, base: tree_sitter::Point, offset: usize) -> Position {
    let mut row = base.row;
    let mut column = base.column;

    for byte in source.as_bytes().iter().take(offset) {
        if *byte == b'\n' {
            row += 1;
            column = 0;
        } else {
            column += 1;
        }
    }

    Position { row, column }
}
