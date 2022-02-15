use std::collections::HashSet;

use crate::{Lint, LintError, ParsedFile};
use crossbeam::channel::Sender;
use once_cell::sync::Lazy;
use tree_sitter::{Query, QueryCursor};

static ITEM_QUERY: Lazy<Query> = Lazy::new(|| {
    Query::new(
        *crate::PHP,
        r#"
        ([
            (class_declaration)
            (trait_declaration)
            (interface_declaration)
        ] @trait)
        "#,
    )
    .unwrap()
});

#[derive(Debug)]
pub struct MultipleItems;

impl Lint for MultipleItems {
    fn should_apply(&self, disabled_lints: &HashSet<&str>) -> bool {
        !disabled_lints.contains(&"multiple-items")
    }

    fn check(&self, parsed: &ParsedFile, emit_error: &Sender<LintError>) {
        let mut cursor = QueryCursor::new();

        let matches = cursor.matches(
            &ITEM_QUERY,
            parsed.tree.root_node(),
            parsed.source.as_bytes(),
        );

        for object in matches.skip(1) {
            let error = parsed.error(&object.captures[0].node, format!("multiple items"));
            emit_error.send(error).unwrap();
        }
    }
}
