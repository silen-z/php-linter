use crossbeam::channel::Sender;
use jwalk::DirEntry;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::fs::read_to_string;
use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};
use tree_sitter::{Node, Parser, Query, QueryCursor, QueryMatch, Tree};

pub mod lints;

pub static PHP: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_php::language());

pub struct ParsedFile {
    pub path: PathBuf,
    pub source: String,
    pub tree: Tree,
}

impl ParsedFile {
    pub fn parse(path: PathBuf, parser: &mut Parser) -> Result<Self> {
        let source = read_to_string(&path).unwrap();

        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| eyre!("Parsing of file {} failed", path.display()))?;

        Ok(Self {
            source,
            tree,
            path: path.to_owned(),
        })
    }

    fn error(&self, node: &Node, message: String) -> LintError {
        LintError {
            message,
            file: self.path.clone(),
            line: node.start_position().row + 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LintError {
    message: String,
    file: PathBuf,
    line: usize,
}

impl PartialOrd for LintError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LintError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file
            .cmp(&other.file)
            .then_with(|| self.line.cmp(&other.line))
    }
}

impl Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message,
            self.file.display(),
            self.line
        )
    }
}

impl std::error::Error for LintError {}

pub trait Lint: Debug {
    fn should_apply(&self, disabled_lints: &HashSet<&str>) -> bool;
    fn check(&self, parsed: &ParsedFile, emit_error: &Sender<LintError>);
}

pub fn procces(entry: DirEntry<((), ())>, emit_error: Sender<LintError>) {
    let mut parser = Parser::new();
    parser.set_language(*PHP).unwrap();

    // parser.set_logger(Some(Box::new(|log_type, msg | {
    //     println!("log_type: {:?} msg {:?}", log_type, msg)
    // })));

    let parsed = ParsedFile::parse(entry.path(), &mut parser).unwrap();

    // for node in traverse_tree(&parsed.tree, tree_sitter_traversal::Order::Post) {
    //     if node.has_error() {
    //         let text = node.utf8_text(&parsed.source.as_bytes())?;

    //         let Point { row, column } = node.start_position();

    //         bail!(
    //             "Error parsing file {}:{}:{}, {}",
    //             path.display(),
    //             row,
    //             column,
    //             text
    //         );
    //     }
    // }

    let disabled_lints = get_disabled_lints(&parsed);

    tracing::debug!(file = %entry.path().display(), ?disabled_lints, "linting");

    for lint in lints::LINTS
        .iter()
        .filter(|lint| lint.should_apply(&disabled_lints))
    {
        lint.check(&parsed, &emit_error)
    }
}

pub fn should_lint(entry: &DirEntry<((), ())>) -> bool {
    entry.file_type.is_file()
        && entry
            .file_name
            .to_str()
            .map_or(false, |f| f.ends_with(".php"))
}

pub fn get_disabled_lints(parsed: &ParsedFile) -> HashSet<&str> {
    let file_disables_query = Query::new(*PHP, "(program (php_tag) (comment)* @comment)").unwrap();

    let mut cursor = QueryCursor::new();

    let mut matches = cursor.matches(
        &file_disables_query,
        parsed.tree.root_node(),
        parsed.source.as_bytes(),
    );

    let mut disabled_lints = HashSet::new();

    if let Some(QueryMatch { captures, .. }) = matches.next() {
        for comment in captures {
            let text = &parsed.source[comment.node.byte_range()];

            if let Some((_, list)) = text.split_once("linter-ignore:") {
                for item in list.split(",") {
                    disabled_lints.insert(item.trim());
                }
            }
        }
    }

    disabled_lints
}
