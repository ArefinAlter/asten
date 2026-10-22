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
            body: to_html(project_name, &included),
        }),
        CompileFormat::Docx => {
            let bytes = to_docx(project_name, &included)?;
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

fn to_html(title: &str, documents: &[&Document]) -> String {
    let mut body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"/><title>{}</title></head><body>",
        escape_xml(title)
    );
    body.push_str(&format!("<h1>{}</h1>", escape_xml(title)));
    for document in documents {
        body.push_str(&format!("<h2>{}</h2>", escape_xml(&document.title)));
        for paragraph in document_plain_text(&document.content_json).split("\n\n") {
            body.push_str(&format!("<p>{}</p>", escape_xml(paragraph)));
        }
    }
    body.push_str("</body></html>");
    body
}

fn to_docx(title: &str, documents: &[&Document]) -> Result<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::{ZipWriter, write::SimpleFileOptions};

    let mut paragraphs = String::new();
    paragraphs.push_str(&docx_paragraph(title, true));
    for document in documents {
        paragraphs.push_str(&docx_paragraph(&document.title, true));
        let text = document_plain_text(&document.content_json);
        if text.is_empty() {
            paragraphs.push_str(&docx_paragraph("", false));
        } else {
            for block in text.split("\n\n") {
                paragraphs.push_str(&docx_paragraph(block, false));
            }
        }
    }

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {paragraphs}
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    let document_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#;

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", options)
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.write_all(content_types.as_bytes())
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.start_file("_rels/.rels", options)
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.write_all(rels.as_bytes())
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.start_file("word/document.xml", options)
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.write_all(document_xml.as_bytes())
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.start_file("word/_rels/document.xml.rels", options)
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.write_all(document_rels.as_bytes())
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        zip.finish()
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
    }
    Ok(cursor.into_inner())
}

fn docx_paragraph(text: &str, heading: bool) -> String {
    let style = if heading {
        r#"<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>"#
    } else {
        ""
    };
    format!(
        "<w:p>{style}<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape_xml(text)
    )
}

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
