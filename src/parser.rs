use std::fmt;

use tree_sitter::{Node, Parser, Point, Tree};

use crate::diagnostic::{Position, Range};

#[derive(Debug)]
pub enum ParseSetupError {
    Smarty(tree_sitter::LanguageError),
    Html(tree_sitter::LanguageError),
    ParseReturnedNone(&'static str),
}

impl fmt::Display for ParseSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseSetupError::Smarty(error) => write!(f, "failed to load Smarty grammar: {error}"),
            ParseSetupError::Html(error) => write!(f, "failed to load HTML grammar: {error}"),
            ParseSetupError::ParseReturnedNone(language) => {
                write!(f, "{language} parser did not return a syntax tree")
            }
        }
    }
}

impl std::error::Error for ParseSetupError {}

pub fn parse_smarty(source: &str) -> Result<Tree, ParseSetupError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_smarty2::LANGUAGE.into())
        .map_err(ParseSetupError::Smarty)?;
    parser
        .parse(source, None)
        .ok_or(ParseSetupError::ParseReturnedNone("Smarty"))
}

pub fn parse_html(source: &str) -> Result<Tree, ParseSetupError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_html::LANGUAGE.into())
        .map_err(ParseSetupError::Html)?;
    parser
        .parse(source, None)
        .ok_or(ParseSetupError::ParseReturnedNone("HTML"))
}

pub fn range_from_node(node: Node) -> Range {
    Range {
        start: position_from_point(node.start_position()),
        end: position_from_point(node.end_position()),
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
    }
}

pub fn offset_range(node: Node, base_byte: usize, base_position: Point) -> Range {
    Range {
        start: offset_position(node.start_position(), base_position),
        end: offset_position(node.end_position(), base_position),
        start_byte: base_byte + node.start_byte(),
        end_byte: base_byte + node.end_byte(),
    }
}

fn position_from_point(point: Point) -> Position {
    Position {
        row: point.row,
        column: point.column,
    }
}

fn offset_position(point: Point, base: Point) -> Position {
    if point.row == 0 {
        Position {
            row: base.row,
            column: base.column + point.column,
        }
    } else {
        Position {
            row: base.row + point.row,
            column: point.column,
        }
    }
}
