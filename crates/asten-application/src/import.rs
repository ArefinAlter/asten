use asten_domain::{
    DocumentKind, fountain_to_prosemirror, markdown_to_prosemirror, plaintext_to_prosemirror,
};
use serde::{Deserialize, Serialize};

use crate::fdx::fdx_to_prosemirror;
use crate::{ApplicationError, Result};

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

pub fn imported_document_json(request: &ImportRequest) -> Result<serde_json::Value> {
    let text = decode_body(request)?;
    match request.format {
        ImportFormat::Markdown | ImportFormat::Html => Ok(markdown_to_prosemirror(
            &strip_html_if_needed(&text, &request.format),
        )),
        ImportFormat::Fountain => Ok(fountain_to_prosemirror(&text)),
        ImportFormat::Plain => Ok(plaintext_to_prosemirror(&text)),
        ImportFormat::Docx => Ok(plaintext_to_prosemirror(&extract_docx_text(&text)?)),
        ImportFormat::Fdx => fdx_to_prosemirror(&text),
    }
}

fn decode_body(request: &ImportRequest) -> Result<String> {
    let encoding = request
        .encoding
        .as_deref()
        .unwrap_or("utf-8")
        .to_ascii_lowercase();
    if encoding == "base64" {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let bytes = STANDARD
            .decode(request.body.trim())
            .map_err(|error| ApplicationError::Validation(error.to_string()))?;
        if matches!(request.format, ImportFormat::Docx) {
            return Ok(STANDARD.encode(bytes));
        }
        String::from_utf8(bytes).map_err(|error| ApplicationError::Validation(error.to_string()))
    } else {
        Ok(request.body.clone())
    }
}

fn strip_html_if_needed(text: &str, format: &ImportFormat) -> String {
    if matches!(format, ImportFormat::Html) {
        let mut stripped = String::new();
        let mut in_tag = false;
        for ch in text.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => stripped.push(ch),
                _ => {}
            }
        }
        stripped
    } else {
        text.to_owned()
    }
}

fn extract_docx_text(base64_or_xml: &str) -> Result<String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use std::io::Cursor;
    let bytes = STANDARD
        .decode(base64_or_xml.trim())
        .or_else(|_| {
            if base64_or_xml.as_bytes().starts_with(b"PK") {
                Ok(base64_or_xml.as_bytes().to_vec())
            } else {
                Err(base64::DecodeError::InvalidByte(0, 0))
            }
        })
        .map_err(|_| ApplicationError::Validation("docx body must be base64".into()))?;
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|error| ApplicationError::Validation(error.to_string()))?;
    let mut file = archive
        .by_name("word/document.xml")
        .map_err(|_| ApplicationError::Validation("docx is missing word/document.xml".into()))?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml)
        .map_err(|error| ApplicationError::Validation(error.to_string()))?;
    Ok(xml_to_text(&xml))
}

fn xml_to_text(xml: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if !text.ends_with('\n') && !text.ends_with(' ') {
                    text.push(' ');
                }
            }
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
