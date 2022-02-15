use crate::Lint;

mod no_long_array_syntax;
mod no_assignment_inside_condition;
mod multiple_items;

pub const LINTS: &[&dyn Lint] = &[
    &multiple_items::MultipleItems,
    &no_long_array_syntax::NoLongArraySyntax,
    &no_assignment_inside_condition::NoAssignmentInsideCondition,
];
