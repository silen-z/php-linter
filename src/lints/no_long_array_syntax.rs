use crossbeam::channel::Sender;
use once_cell::sync::Lazy;
use tree_sitter::{Query, QueryCursor};

use crate::{Lint, LintError};

static ARRAY_QUERY: Lazy<Query> =
    Lazy::new(|| Query::new(*crate::PHP, r#"(array_creation_expression "array") @a"#).unwrap());

#[derive(Debug)]
pub struct NoLongArraySyntax;

impl Lint for NoLongArraySyntax {
    fn should_apply(&self, disabled_lints: &std::collections::HashSet<&str>) -> bool {
        !disabled_lints.contains("no-long-array-syntax")
    }

    fn check(&self, parsed: &crate::ParsedFile, emit_error: &Sender<LintError>) {
        let mut cursor = QueryCursor::new();

        let matches = cursor.matches(
            &ARRAY_QUERY,
            parsed.tree.root_node(),
            parsed.source.as_bytes(),
        );

        for object in matches {
            let error = parsed.error(&object.captures[0].node, format!("long array syntax"));
            emit_error.send(error).unwrap();
        }
    }
}
