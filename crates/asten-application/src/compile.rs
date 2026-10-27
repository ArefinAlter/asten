use std::collections::{HashMap, HashSet};

use asten_domain::{
    CompileArtifact, Document, DocumentId, document_plain_text, prosemirror_to_fountain,
    prosemirror_to_markdown,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::epub::to_epub;
use crate::fdx::to_fdx;
use crate::{ApplicationError, Result};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CompileFormat {
    #[default]
    Markdown,
    Plain,
    Fountain,
    Html,
    Docx,
    Epub,
    Fdx,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CompileRequest {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub format: CompileFormat,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub document_ids: Vec<asten_domain::DocumentId>,
}

pub fn compile_documents(
    project_name: &str,
    documents: &[Document],
    format: &CompileFormat,
) -> Result<CompileArtifact> {
    let ordered = binder_order(documents);
    let included: Vec<&Document> = ordered.iter().map(|(_, document)| *document).collect();
    if included.is_empty() {
        return Err(ApplicationError::Validation(
            "nothing to compile — include a scene, chapter, or manuscript".into(),
        ));
    }
    let slug = slugify(project_name);
    match format {
        CompileFormat::Markdown => Ok(CompileArtifact {
            filename: format!("{slug}.md"),
            mime: "text/markdown".into(),
            encoding: "utf-8".into(),
            body: to_markdown(project_name, &included),
        }),
        CompileFormat::Plain => Ok(CompileArtifact {
            filename: format!("{slug}.txt"),
            mime: "text/plain".into(),
            encoding: "utf-8".into(),
            body: to_plain(project_name, &included),
        }),
        CompileFormat::Fountain => Ok(CompileArtifact {
            filename: format!("{slug}.fountain"),
            mime: "text/plain".into(),
            encoding: "utf-8".into(),
            body: to_fountain(project_name, &included),
        }),
        CompileFormat::Html => Ok(CompileArtifact {
            filename: format!("{slug}.html"),
            mime: "text/html".into(),
            encoding: "utf-8".into(),
            body: to_html(project_name, &ordered),
        }),
        CompileFormat::Docx => {
            let bytes = crate::docx_out::to_docx(project_name, &ordered)?;
            Ok(CompileArtifact {
                filename: format!("{slug}.docx"),
                mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .into(),
                encoding: "base64".into(),
                body: STANDARD.encode(bytes),
            })
        }
        CompileFormat::Epub => {
            let bytes = to_epub(project_name, &ordered)?;
            Ok(CompileArtifact {
                filename: format!("{slug}.epub"),
                mime: "application/epub+zip".into(),
                encoding: "base64".into(),
                body: STANDARD.encode(bytes),
            })
        }
        CompileFormat::Fdx => Ok(CompileArtifact {
            filename: format!("{slug}.fdx"),
            mime: "application/xml".into(),
            encoding: "utf-8".into(),
            body: to_fdx(project_name, &included),
        }),
    }
}

/// The documents that compile, in binder order: a depth-first walk of the
/// project tree with siblings by position then creation time, the order the
/// desk's binder shows. Only prose kinds not excluded from compile are kept.
/// The depth counts kept ancestors, so a scene under an excluded or non-prose
/// parent sits one level up rather than vanishing. A document whose parent is
/// not in the set (a filtered compile, or an orphan) is a root.
pub(crate) fn binder_order(documents: &[Document]) -> Vec<(usize, &Document)> {
    let ids: HashSet<DocumentId> = documents.iter().map(|document| document.id).collect();
    let mut children: HashMap<Option<DocumentId>, Vec<&Document>> = HashMap::new();
    for document in documents {
        let parent = document.parent_id.filter(|parent| ids.contains(parent));
        children.entry(parent).or_default().push(document);
    }
    for siblings in children.values_mut() {
        siblings.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then(a.created_at.cmp(&b.created_at))
                .then(a.id.cmp(&b.id))
        });
    }
    let mut ordered = Vec::new();
    let mut visited = HashSet::new();
    walk_binder(None, 0, &children, &mut visited, &mut ordered);
    // Documents inside a parent cycle are reachable from no root; keep them
    // rather than lose prose, in storage order at the top level.
    for document in documents {
        if !visited.contains(&document.id) && compiles(document) {
            ordered.push((0, document));
        }
    }
    ordered
}

fn compiles(document: &Document) -> bool {
    !document.exclude_from_compile && document.kind.is_prose()
}

fn walk_binder<'a>(
    parent: Option<DocumentId>,
    depth: usize,
    children: &HashMap<Option<DocumentId>, Vec<&'a Document>>,
    visited: &mut HashSet<DocumentId>,
    ordered: &mut Vec<(usize, &'a Document)>,
) {
    let Some(siblings) = children.get(&parent) else {
        return;
    };
    for document in siblings {
        if !visited.insert(document.id) {
            continue;
        }
        let next_depth = if compiles(document) {
            ordered.push((depth, *document));
            depth + 1
        } else {
            depth
        };
        walk_binder(Some(document.id), next_depth, children, visited, ordered);
    }
}

fn to_markdown(title: &str, documents: &[&Document]) -> String {
    let mut body = format!("# {title}\n\n");
    for document in documents {
        body.push_str(&format!("## {}\n\n", document.title));
        body.push_str(&prosemirror_to_markdown(&document.content_json));
        body.push('\n');
    }
    body
}

fn to_plain(title: &str, documents: &[&Document]) -> String {
    let mut body = format!("{title}\n\n");
    for document in documents {
        body.push_str(&document.title);
        body.push_str("\n\n");
        body.push_str(&document_plain_text(&document.content_json));
        body.push_str("\n\n");
    }
    body
}

fn to_fountain(title: &str, documents: &[&Document]) -> String {
    let mut body = format!("Title: {title}\nCredit: Written by\n\n");
    for document in documents {
        body.push_str(&prosemirror_to_fountain(
            &document.title,
            &document.content_json,
        ));
        body.push('\n');
    }
    body
}

fn to_html(title: &str, documents: &[(usize, &Document)]) -> String {
    let mut body = format!(
        "<!doctype html>
<html lang=\"en\"><head><meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/><title>{}</title><style>{HTML_STYLE}</style></head><body><main>
",
        escape_xml(title)
    );
    body.push_str(&format!(
        "<h1>{}</h1>
",
        escape_xml(title)
    ));
    for (depth, document) in documents {
        let level = (depth + 2).clamp(2, 6);
        body.push_str(&format!(
            "<section>
<h{level}>{}</h{level}>
",
            escape_xml(&document.title)
        ));
        if let Some(nodes) = document
            .content_json
            .get("content")
            .and_then(serde_json::Value::as_array)
        {
            crate::epub::write_blocks(nodes, &mut body, depth + 1);
        }
        body.push_str(
            "</section>
",
        );
    }
    body.push_str(
        "</main></body></html>
",
    );
    body
}

const HTML_STYLE: &str = "body{margin:0;background:#faf7f0;color:#1d1a16;font:18px/1.7 Georgia,'Times New Roman',serif}main{max-width:40em;margin:0 auto;padding:3em 1.2em}h1,h2,h3,h4{line-height:1.25}h1{text-align:center;margin:2em 0}p{margin:0;text-indent:1.5em}h2+p,h3+p,.scene-break+p,hr+p{text-indent:0}.scene-break{text-align:center;text-indent:0;margin:1.6em 0}blockquote{margin:1em 2em;font-style:italic}ins{text-decoration:none;border-bottom:2px solid #2d7a4f}del{color:#a33}";

/// Text or attribute content for XML: the five special characters escaped
/// and the control characters XML 1.0 forbids dropped, so every part stays
/// well-formed whatever was pasted into the manuscript.
pub(crate) fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\t' | '\n' | '\r' => escaped.push(ch),
            ch if u32::from(ch) < 0x20 || matches!(ch, '\u{FFFE}' | '\u{FFFF}') => {}
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn slugify(value: &str) -> String {
    let slug: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "manuscript".into()
    } else {
        slug.into()
    }
}
