use tree_sitter::LanguageError;

pub use tree_sitter::Node;
pub use tree_sitter::Tree;
pub use tree_sitter::TreeCursor;

pub enum ParseError {
    LanguageError(LanguageError),
}

impl From<LanguageError> for ParseError {
    fn from(err: LanguageError) -> Self {
        Self::LanguageError(err)
    }
}

pub fn parse(
    text: &str,
    old_tree: Option<&tree_sitter::Tree>,
) -> Result<Option<tree_sitter::Tree>, ParseError> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_markdown::language())?;
    let tree = parser.parse(text, old_tree);
    return Ok(tree);
}
