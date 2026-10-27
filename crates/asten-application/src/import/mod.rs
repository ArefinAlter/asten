//! Importing a file into the binder.
//!
//! Six formats: Markdown, plain text, HTML, Fountain, Word (`.docx`) and Final
//! Draft (`.fdx`). The four that carry prose (Markdown, plain text, HTML and
//! Word) are read into a block model first, so headings, emphasis, quotes,
//! lists, scene breaks, tracked changes and comments survive and a long file
//! can be split into chapters; Fountain and Final Draft keep their own readers.

use asten_domain::{DocumentKind, empty_prosemirror_doc, fountain_to_prosemirror};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fdx::fdx_to_prosemirror;
use crate::{ApplicationError, Result};

mod blocks;
mod chapters;
mod docx;
mod html;
mod inline;
mod markdown;

pub(crate) use docx::ImportedComment;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    Markdown,
    Plain,
    Fountain,
    Html,
    Docx,
    Fdx,
}

/// Whether a long file becomes one document or one per chapter.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ImportSplit {
    /// One document, whatever the file holds.
    #[default]
    None,
    /// A manuscript with a document per chapter when the file has at least
    /// three chapter headings; otherwise one document.
    Auto,
    /// The same with at least two chapter headings.
    Chapters,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct ImportRequest {
    pub filename: String,
    pub format: ImportFormat,
    pub body: String,
    #[serde(default)]
    pub encoding: Option<String>,
    pub parent_id: Option<asten_domain::DocumentId>,
    pub kind: Option<DocumentKind>,
    /// Split a long file into chapters. Absent means one document.
    #[serde(default)]
    pub split: Option<ImportSplit>,
}

pub fn imported_title(filename: &str) -> String {
    std::path::Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported")
        .to_owned()
}

/// One document the import will create.
pub(crate) struct PlannedDocument {
    pub title: String,
    pub content: Value,
    pub comments: Vec<ImportedComment>,
}

/// What an import will create.
pub(crate) struct Plan {
    /// Set when the file became several chapters under one manuscript.
    pub container: Option<String>,
    pub documents: Vec<PlannedDocument>,
    pub insertions: usize,
    pub deletions: usize,
}

/// The bytes of the request body: base64 when asked, else the text as UTF-8.
fn body_bytes(request: &ImportRequest) -> Result<Vec<u8>> {
    let encoding = request
        .encoding
        .as_deref()
        .unwrap_or("utf-8")
        .to_ascii_lowercase();
    if encoding == "base64" || matches!(request.format, ImportFormat::Docx) {
        return STANDARD.decode(request.body.trim()).map_err(|error| {
            ApplicationError::Validation(format!("the body must be base64: {error}"))
        });
    }
    Ok(request.body.clone().into_bytes())
}

/// Text from bytes: a byte-order mark names the encoding; otherwise UTF-8, and
/// failing that Windows-1252, which is what most text editors on Windows wrote
/// before UTF-8 was common.
pub(crate) fn decode_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    let utf16 = |rest: &[u8], big: bool| -> String {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|pair| {
                if big {
                    u16::from_be_bytes([pair[0], pair[1]])
                } else {
                    u16::from_le_bytes([pair[0], pair[1]])
                }
            })
            .collect();
        String::from_utf16_lossy(&units)
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(rest, false);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(rest, true);
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => bytes.iter().map(|&b| windows_1252(b)).collect(),
    }
}

fn windows_1252(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '\u{20ac}', '\u{81}', '\u{201a}', '\u{192}', '\u{201e}', '\u{2026}', '\u{2020}',
        '\u{2021}', '\u{2c6}', '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '\u{8d}', '\u{17d}',
        '\u{8f}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}',
        '\u{2014}', '\u{2dc}', '\u{2122}', '\u{161}', '\u{203a}', '\u{153}', '\u{9d}', '\u{17e}',
        '\u{178}',
    ];
    match byte {
        0x80..=0x9f => HIGH[(byte - 0x80) as usize],
        b => b as char,
    }
}

/// Work out what the request will create.
pub(crate) fn plan(request: &ImportRequest) -> Result<Plan> {
    let bytes = body_bytes(request)?;
    let title = imported_title(&request.filename);
    let mode = match request.split.unwrap_or_default() {
        ImportSplit::None => chapters::Mode::Off,
        ImportSplit::Auto => chapters::Mode::Auto,
        ImportSplit::Chapters => chapters::Mode::Chapters,
    };
    let (blocks, comments, insertions, deletions) = match request.format {
        ImportFormat::Fountain => {
            return Ok(single(title, fountain_to_prosemirror(&decode_text(&bytes))));
        }
        ImportFormat::Fdx => {
            return Ok(single(title, fdx_to_prosemirror(&decode_text(&bytes))?));
        }
        ImportFormat::Markdown => (markdown::markdown(&decode_text(&bytes)), Vec::new(), 0, 0),
        ImportFormat::Plain => (markdown::plain(&decode_text(&bytes)), Vec::new(), 0, 0),
        ImportFormat::Html => (html::html(&decode_text(&bytes)), Vec::new(), 0, 0),
        ImportFormat::Docx => {
            let read = docx::read(bytes)?;
            (read.blocks, read.comments, read.insertions, read.deletions)
        }
    };
    if let Some(split) = chapters::split(&blocks, &comments, mode) {
        return Ok(Plan {
            container: Some(split.title.unwrap_or(title)),
            documents: split
                .parts
                .into_iter()
                .map(|part| PlannedDocument {
                    title: part.title,
                    content: blocks::to_prosemirror(&part.blocks),
                    comments: part.comments,
                })
                .collect(),
            insertions,
            deletions,
        });
    }
    Ok(Plan {
        container: None,
        documents: vec![PlannedDocument {
            title,
            content: blocks::to_prosemirror(&blocks),
            comments,
        }],
        insertions,
        deletions,
    })
}

fn single(title: String, content: Value) -> Plan {
    Plan {
        container: None,
        documents: vec![PlannedDocument {
            title,
            content,
            comments: Vec::new(),
        }],
        insertions: 0,
        deletions: 0,
    }
}

/// The page an import would create, as ProseMirror JSON (one document; no
/// splitting).
pub fn imported_document_json(request: &ImportRequest) -> Result<Value> {
    let mut request = request.clone();
    request.split = None;
    let mut plan = plan(&request)?;
    Ok(plan
        .documents
        .pop()
        .map_or_else(empty_prosemirror_doc, |document| document.content))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(format: ImportFormat, body: &str) -> ImportRequest {
        ImportRequest {
            filename: "book.txt".into(),
            format,
            body: body.into(),
            encoding: None,
            parent_id: None,
            kind: None,
            split: Some(ImportSplit::Auto),
        }
    }

    #[test]
    fn a_plain_text_novel_becomes_a_manuscript_of_chapters() {
        let mut text = String::from("A Book\n\nby Someone\n\n");
        for n in 1..=4 {
            text.push_str(&format!(
                "CHAPTER {n}.\n\nThe first line of chapter {n},\nwrapped by hand.\n\nA second paragraph with _emphasis_.\n\n"
            ));
        }
        let plan = plan(&request(ImportFormat::Plain, &text)).unwrap();
        assert_eq!(plan.container.as_deref(), Some("book"));
        let titles: Vec<&str> = plan.documents.iter().map(|d| d.title.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "Front matter",
                "CHAPTER 1.",
                "CHAPTER 2.",
                "CHAPTER 3.",
                "CHAPTER 4."
            ]
        );
        assert_eq!(
            plan.documents[1].content["content"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn a_short_file_stays_one_document_even_when_split_is_asked() {
        let plan = plan(&request(
            ImportFormat::Markdown,
            "# One\n\nonly text\n\n# Two\n\nmore",
        ))
        .unwrap();
        assert!(plan.container.is_none());
        assert_eq!(plan.documents.len(), 1);
    }

    #[test]
    fn no_split_is_the_default_for_old_clients() {
        let mut r = request(
            ImportFormat::Plain,
            "CHAPTER 1.\n\na\n\nCHAPTER 2.\n\nb\n\nCHAPTER 3.\n\nc",
        );
        r.split = None;
        assert!(plan(&r).unwrap().container.is_none());
    }

    #[test]
    fn text_is_decoded_by_its_byte_order_mark_or_as_windows_1252() {
        assert_eq!(decode_text(b"\xEF\xBB\xBFhello"), "hello");
        assert_eq!(decode_text(&[0xFF, 0xFE, b'h', 0, b'i', 0]), "hi");
        assert_eq!(decode_text(&[0xFE, 0xFF, 0, b'h', 0, b'i']), "hi");
        assert_eq!(
            decode_text(b"caf\xE9 \x93quoted\x94"),
            "caf\u{e9} \u{201c}quoted\u{201d}"
        );
        assert_eq!(decode_text("caf\u{e9}".as_bytes()), "caf\u{e9}");
    }

    #[test]
    fn base64_text_and_word_bodies_are_decoded() {
        let mut r = request(
            ImportFormat::Markdown,
            &STANDARD.encode("# Heading\n\nbody"),
        );
        r.encoding = Some("base64".into());
        r.split = None;
        let planned = plan(&r).unwrap();
        assert_eq!(
            planned.documents[0].content["content"][0]["type"],
            "heading"
        );
        let w = request(ImportFormat::Docx, "not base64!!");
        assert!(plan(&w).is_err());
    }

    #[test]
    fn html_and_markdown_keep_emphasis() {
        let html = plan(&request(ImportFormat::Html, "<p>a <em>b</em></p>")).unwrap();
        assert_eq!(
            html.documents[0].content["content"][0]["content"][1]["marks"][0]["type"],
            "italic"
        );
        let md = plan(&request(ImportFormat::Markdown, "a *b*")).unwrap();
        assert_eq!(
            md.documents[0].content["content"][0]["content"][1]["marks"][0]["type"],
            "italic"
        );
    }
}
