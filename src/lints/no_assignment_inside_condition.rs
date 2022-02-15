use crossbeam::channel::Sender;
use once_cell::sync::Lazy;
use tree_sitter::{Query, QueryCursor};

use crate::{Lint, LintError};

static CONDITIONS: Lazy<Query> =
    Lazy::new(|| Query::new(*crate::PHP, r#"(if_statement condition: (_) @a)"#).unwrap());

static ASSIGNMENTS: Lazy<Query> =
    Lazy::new(|| Query::new(*crate::PHP, r#"(assignment_expression) @a"#).unwrap());

#[derive(Debug)]
pub struct NoAssignmentInsideCondition;

impl Lint for NoAssignmentInsideCondition {
    fn should_apply(&self, disabled_lints: &std::collections::HashSet<&str>) -> bool {
        !disabled_lints.contains("no-assignment-inside-condition")
    }

    fn check(&self, parsed: &crate::ParsedFile, emit_error: &Sender<LintError>) {
        let mut cursor = QueryCursor::new();

        let matches = cursor.matches(
            &CONDITIONS,
            parsed.tree.root_node(),
            parsed.source.as_bytes(),
        );

        for condition in matches {
            let mut cursor = QueryCursor::new();

            let mut assignments = cursor.matches(
                &ASSIGNMENTS,
                condition.captures[0].node,
                parsed.source.as_bytes(),
            );

            if let Some(assignment) = assignments.next() {
                let error =
                    parsed.error(&assignment.captures[0].node, format!("assignment syntax in condition"));
                emit_error.send(error).unwrap();
            }
        }
    }
}
