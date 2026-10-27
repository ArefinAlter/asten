//! What the importers produce before it becomes a ProseMirror document: a
//! small block model (paragraphs, headings, quotes, lists, rules) whose text is
//! made of spans with marks. Every importer fills this; one function turns it
//! into the editor's JSON, with a stable id on every block.

use asten_domain::empty_prosemirror_doc;
use serde_json::{Value, json};
use uuid::Uuid;

/// Marks in the order they are written on a text node.
pub(crate) const MARK_ORDER: [&str; 7] = [
    "bold",
    "italic",
    "underline",
    "strike",
    "code",
    "insertion",
    "deletion",
];

/// Text with one set of marks.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Span {
    pub text: String,
    pub marks: Vec<&'static str>,
}

/// One piece of a paragraph.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Inline {
    Text(Span),
    Break,
}

/// A block of the page.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Block {
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    Quote(Vec<Block>),
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
    },
    Rule,
    SceneBreak,
}

/// Plain text with no marks.
#[cfg(test)]
pub(crate) fn text(value: &str) -> Inline {
    Inline::Text(Span {
        text: value.to_owned(),
        marks: Vec::new(),
    })
}

/// Text with marks, kept in the canonical order.
pub(crate) fn marked(value: &str, marks: &[&'static str]) -> Inline {
    let mut ordered: Vec<&'static str> = MARK_ORDER
        .iter()
        .copied()
        .filter(|mark| marks.contains(mark))
        .collect();
    ordered.dedup();
    Inline::Text(Span {
        text: value.to_owned(),
        marks: ordered,
    })
}

/// Join neighbours that carry the same marks, drop empty text, and trim the
/// ends of the line.
pub(crate) fn tidy(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Text(span) if span.text.is_empty() => {}
            Inline::Text(span) => match out.last_mut() {
                Some(Inline::Text(last)) if last.marks == span.marks => {
                    last.text.push_str(&span.text)
                }
                _ => out.push(Inline::Text(span)),
            },
            Inline::Break => {
                if !matches!(out.last(), None | Some(Inline::Break)) {
                    out.push(Inline::Break);
                }
            }
        }
    }
    while matches!(out.last(), Some(Inline::Break)) {
        out.pop();
    }
    if let Some(Inline::Text(first)) = out.first_mut() {
        first.text = first.text.trim_start().to_owned();
    }
    if let Some(Inline::Text(last)) = out.last_mut() {
        last.text = last.text.trim_end().to_owned();
    }
    out.retain(|inline| !matches!(inline, Inline::Text(span) if span.text.is_empty()));
    out
}

/// The text of a run of inlines, with a space for a break.
pub(crate) fn inline_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(span) => out.push_str(&span.text),
            Inline::Break => out.push('\n'),
        }
    }
    out
}

/// The plain text of a block.
pub(crate) fn block_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) | Block::Heading(_, inlines) => inline_text(inlines),
        Block::Quote(blocks) => blocks.iter().map(block_text).collect::<Vec<_>>().join("\n"),
        Block::List { items, .. } => items
            .iter()
            .flat_map(|item| item.iter().map(block_text))
            .collect::<Vec<_>>()
            .join("\n"),
        Block::Rule | Block::SceneBreak => String::new(),
    }
}

fn id() -> Value {
    json!({ "id": Uuid::now_v7().to_string() })
}

fn inline_nodes(inlines: &[Inline]) -> Vec<Value> {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::Break => json!({ "type": "hardBreak" }),
            Inline::Text(span) => {
                let mut node = json!({ "type": "text", "text": span.text });
                if !span.marks.is_empty() {
                    node["marks"] =
                        Value::Array(span.marks.iter().map(|m| json!({ "type": m })).collect());
                }
                node
            }
        })
        .collect()
}

fn node(block: &Block) -> Value {
    match block {
        Block::Paragraph(inlines) => json!({
            "type": "paragraph", "attrs": id(), "content": inline_nodes(inlines),
        }),
        Block::Heading(level, inlines) => json!({
            "type": "heading",
            "attrs": { "id": Uuid::now_v7().to_string(), "level": (*level).clamp(1, 6) },
            "content": inline_nodes(inlines),
        }),
        Block::Quote(blocks) => json!({
            "type": "blockquote", "attrs": id(),
            "content": blocks.iter().map(node).collect::<Vec<_>>(),
        }),
        Block::List { ordered, items } => json!({
            "type": if *ordered { "orderedList" } else { "bulletList" },
            "attrs": id(),
            "content": items.iter().map(|item| json!({
                "type": "listItem", "attrs": id(),
                "content": item.iter().map(node).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }),
        Block::Rule => json!({ "type": "horizontalRule", "attrs": id() }),
        Block::SceneBreak => json!({ "type": "sceneBreak", "attrs": id() }),
    }
}

/// The editor's document for these blocks.
pub(crate) fn to_prosemirror(blocks: &[Block]) -> Value {
    let content: Vec<Value> = blocks.iter().map(node).collect();
    if content.is_empty() {
        return empty_prosemirror_doc();
    }
    json!({ "type": "doc", "content": content })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tidy_joins_same_marks_and_trims() {
        let tidied = tidy(vec![
            text("  Hello "),
            text("world "),
            marked("again", &["italic"]),
            text("  "),
        ]);
        assert_eq!(
            tidied,
            vec![text("Hello world "), marked("again", &["italic"])]
        );
    }

    #[test]
    fn blocks_become_nodes_with_ids_and_marks() {
        let doc = to_prosemirror(&[
            Block::Heading(2, vec![text("One")]),
            Block::Paragraph(vec![text("She "), marked("never", &["bold", "italic"])]),
            Block::List {
                ordered: false,
                items: vec![vec![Block::Paragraph(vec![text("a")])]],
            },
            Block::SceneBreak,
        ]);
        let content = doc["content"].as_array().unwrap();
        assert_eq!(content[0]["attrs"]["level"], 2);
        assert!(content[0]["attrs"]["id"].as_str().unwrap().len() > 20);
        assert_eq!(content[1]["content"][1]["marks"][0]["type"], "bold");
        assert_eq!(content[1]["content"][1]["marks"][1]["type"], "italic");
        assert_eq!(content[2]["type"], "bulletList");
        assert_eq!(content[2]["content"][0]["type"], "listItem");
        assert_eq!(content[3]["type"], "sceneBreak");
    }

    #[test]
    fn an_empty_import_is_an_empty_page() {
        let doc = to_prosemirror(&[]);
        assert_eq!(doc["content"][0]["type"], "paragraph");
    }
}
