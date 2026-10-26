//! Compile to Word (`.docx`).
//!
//! A real package: styles (Normal, Title, Heading 1 to 6, Quote, List
//! Paragraph, Scene Break, Code), numbering for bullet and numbered lists,
//! document properties, and the manuscript's own formatting: italic, bold,
//! underline and strike-through runs, scene breaks, block quotes, lists, and
//! tracked insertions and deletions written as Word's `w:ins` and `w:del`, so
//! Word shows them as tracked changes. Opens in Word, LibreOffice and
//! python-docx.

use std::io::{Cursor, Write};

use asten_domain::Document;
use chrono::{SecondsFormat, Utc};
use serde_json::Value;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{ApplicationError, Result, compile::escape_xml};

struct Out {
    body: String,
    revision: u32,
    date: String,
    /// One numbering instance per numbered list, so each starts at 1.
    ordered_lists: u32,
}

impl Out {
    fn next_revision(&mut self) -> u32 {
        self.revision += 1;
        self.revision
    }
}

const FIRST_ORDERED_NUM: u32 = 10;

fn paragraph(style: Option<&str>, extra: &str, content: &str) -> String {
    let props = match (style, extra) {
        (None, "") => String::new(),
        (Some(style), "") => format!("<w:pPr><w:pStyle w:val=\"{style}\"/></w:pPr>"),
        (None, extra) => format!("<w:pPr>{extra}</w:pPr>"),
        (Some(style), extra) => format!("<w:pPr><w:pStyle w:val=\"{style}\"/>{extra}</w:pPr>"),
    };
    format!("<w:p>{props}{content}</w:p>")
}

fn run(text: &str, props: &str, deleted: bool) -> String {
    let rpr = if props.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{props}</w:rPr>")
    };
    let tag = if deleted { "w:delText" } else { "w:t" };
    format!(
        "<w:r>{rpr}<{tag} xml:space=\"preserve\">{}</{tag}></w:r>",
        escape_xml(text)
    )
}

fn inline(nodes: &[Value], out: &mut Out) -> String {
    let mut xml = String::new();
    for node in nodes {
        match node.get("type").and_then(Value::as_str) {
            Some("hardBreak") => xml.push_str("<w:r><w:br/></w:r>"),
            _ => {
                if let Some(text) = node.get("text").and_then(Value::as_str) {
                    let marks: Vec<&str> = node
                        .get("marks")
                        .and_then(Value::as_array)
                        .map(|marks| {
                            marks
                                .iter()
                                .filter_map(|m| m.get("type").and_then(Value::as_str))
                                .collect()
                        })
                        .unwrap_or_default();
                    let has = |names: &[&str]| marks.iter().any(|m| names.contains(m));
                    let mut props = String::new();
                    if has(&["bold", "strong"]) {
                        props.push_str("<w:b/>");
                    }
                    if has(&["italic", "em"]) {
                        props.push_str("<w:i/>");
                    }
                    if has(&["strike"]) {
                        props.push_str("<w:strike/>");
                    }
                    if has(&["underline"]) {
                        props.push_str("<w:u w:val=\"single\"/>");
                    }
                    if has(&["code"]) {
                        props.push_str("<w:rStyle w:val=\"CodeChar\"/>");
                    }
                    let attrs = |id: u32, date: &str| {
                        format!("w:id=\"{id}\" w:author=\"Asten\" w:date=\"{date}\"")
                    };
                    let mut piece = run(text, &props, has(&["deletion"]));
                    if has(&["deletion"]) {
                        let id = out.next_revision();
                        piece = format!("<w:del {}>{piece}</w:del>", attrs(id, &out.date));
                    }
                    if has(&["insertion"]) {
                        let id = out.next_revision();
                        piece = format!("<w:ins {}>{piece}</w:ins>", attrs(id, &out.date));
                    }
                    xml.push_str(&piece);
                } else if let Some(children) = node.get("content").and_then(Value::as_array) {
                    xml.push_str(&inline(children, out));
                }
            }
        }
    }
    xml
}

fn children(node: &Value) -> &[Value] {
    node.get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn blocks(nodes: &[Value], depth: usize, quote: bool, list: Option<(u32, usize)>, out: &mut Out) {
    for node in nodes {
        block(node, depth, quote, list, out);
    }
}

fn block(node: &Value, depth: usize, quote: bool, list: Option<(u32, usize)>, out: &mut Out) {
    let kind = node.get("type").and_then(Value::as_str).unwrap_or_default();
    match kind {
        "paragraph" | "text" | "hardBreak" => {
            let content = if kind == "paragraph" {
                inline(children(node), out)
            } else {
                inline(std::slice::from_ref(node), out)
            };
            let xml = match (list, quote) {
                (Some((num, level)), _) => paragraph(
                    Some("ListParagraph"),
                    &format!(
                        "<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"{num}\"/></w:numPr>"
                    ),
                    &content,
                ),
                (None, true) => paragraph(Some("Quote"), "", &content),
                (None, false) => paragraph(None, "", &content),
            };
            out.body.push_str(&xml);
        }
        "heading" => {
            let level = node
                .pointer("/attrs/level")
                .and_then(Value::as_u64)
                .unwrap_or(1) as usize;
            let style = format!("Heading{}", (level + depth + 1).clamp(1, 6));
            let content = inline(children(node), out);
            out.body.push_str(&paragraph(Some(&style), "", &content));
        }
        "blockquote" => blocks(children(node), depth, true, list, out),
        "bulletList" | "orderedList" => {
            let level = list.map_or(0, |(_, level)| level + 1).min(8);
            let num = if kind == "bulletList" {
                1
            } else {
                out.ordered_lists += 1;
                FIRST_ORDERED_NUM + out.ordered_lists
            };
            for item in children(node) {
                // A list item holds blocks; the first paragraph carries the number.
                let mut first = true;
                for inner in children(item) {
                    let inner_kind = inner
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    if matches!(inner_kind, "bulletList" | "orderedList") {
                        block(inner, depth, quote, Some((num, level)), out);
                    } else if first {
                        block(inner, depth, quote, Some((num, level)), out);
                        first = false;
                    } else {
                        block(inner, depth, false, None, out);
                    }
                }
            }
        }
        "codeBlock" => {
            let content = inline(children(node), out);
            out.body.push_str(&paragraph(Some("Code"), "", &content));
        }
        "horizontalRule" | "sceneBreak" => {
            out.body
                .push_str(&paragraph(Some("SceneBreak"), "", &run("* * *", "", false)));
        }
        _ => {
            if children(node).is_empty() {
                if let Some(text) = node.get("text").and_then(Value::as_str) {
                    out.body
                        .push_str(&paragraph(None, "", &run(text, "", false)));
                }
            } else {
                blocks(children(node), depth, quote, list, out);
            }
        }
    }
}

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="Times New Roman" w:cs="Times New Roman"/><w:sz w:val="24"/><w:szCs w:val="24"/><w:lang w:val="en-GB"/></w:rPr></w:rPrDefault>
    <w:pPrDefault><w:pPr><w:spacing w:after="0" w:line="360" w:lineRule="auto"/></w:pPr></w:pPrDefault>
  </w:docDefaults>
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:pPr><w:ind w:firstLine="720"/></w:pPr></w:style>
  <w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="2400" w:after="480"/><w:ind w:firstLine="0"/><w:jc w:val="center"/></w:pPr><w:rPr><w:b/><w:sz w:val="48"/><w:szCs w:val="48"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:pageBreakBefore/><w:spacing w:before="480" w:after="240"/><w:ind w:firstLine="0"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="36"/><w:szCs w:val="36"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="360" w:after="160"/><w:ind w:firstLine="0"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="30"/><w:szCs w:val="30"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="280" w:after="120"/><w:ind w:firstLine="0"/><w:outlineLvl w:val="2"/></w:pPr><w:rPr><w:b/><w:i/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:ind w:firstLine="0"/><w:outlineLvl w:val="3"/></w:pPr><w:rPr><w:b/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:ind w:firstLine="0"/><w:outlineLvl w:val="4"/></w:pPr><w:rPr><w:i/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:ind w:firstLine="0"/><w:outlineLvl w:val="5"/></w:pPr><w:rPr><w:i/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:before="120" w:after="120"/><w:ind w:left="720" w:right="720" w:firstLine="0"/></w:pPr><w:rPr><w:i/></w:rPr></w:style>
  <w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:ind w:left="720" w:firstLine="0"/></w:pPr></w:style>
  <w:style w:type="paragraph" w:styleId="SceneBreak"><w:name w:val="Scene Break"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:before="240" w:after="240"/><w:ind w:firstLine="0"/><w:jc w:val="center"/></w:pPr></w:style>
  <w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:firstLine="0"/></w:pPr><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/><w:sz w:val="20"/></w:rPr></w:style>
  <w:style w:type="character" w:styleId="CodeChar"><w:name w:val="Code Char"/><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas" w:cs="Consolas"/></w:rPr></w:style>
</w:styles>"#;

fn numbering(ordered_lists: u32) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="1"><w:multiLevelType w:val="hybridMultilevel"/>"#,
    );
    for level in 0..9 {
        xml.push_str(&format!(
            "<w:lvl w:ilvl=\"{level}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/><w:lvlText w:val=\"\u{2022}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            720 + level * 360
        ));
    }
    xml.push_str("</w:abstractNum><w:abstractNum w:abstractNumId=\"2\"><w:multiLevelType w:val=\"hybridMultilevel\"/>");
    for level in 0..9 {
        xml.push_str(&format!(
            "<w:lvl w:ilvl=\"{level}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/><w:lvlText w:val=\"%{}.\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            level + 1,
            720 + level * 360
        ));
    }
    xml.push_str("</w:abstractNum><w:num w:numId=\"1\"><w:abstractNumId w:val=\"1\"/></w:num>");
    for n in 1..=ordered_lists {
        xml.push_str(&format!(
            "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"2\"/><w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"1\"/></w:lvlOverride></w:num>",
            FIRST_ORDERED_NUM + n
        ));
    }
    xml.push_str("</w:numbering>");
    xml
}

/// The manuscript as a `.docx`. `documents` are in binder order with their depth.
pub(crate) fn to_docx(title: &str, documents: &[(usize, &Document)]) -> Result<Vec<u8>> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut out = Out {
        body: String::new(),
        revision: 0,
        date: now.clone(),
        ordered_lists: 0,
    };
    out.body
        .push_str(&paragraph(Some("Title"), "", &run(title, "", false)));
    for (depth, document) in documents {
        out.body.push_str(&paragraph(
            Some(&format!("Heading{}", (depth + 1).clamp(1, 6))),
            "",
            &run(&document.title, "", false),
        ));
        let content = children(&document.content_json);
        if content.is_empty() {
            out.body.push_str(&paragraph(None, "", ""));
        } else {
            blocks(content, *depth, false, None, &mut out);
        }
    }
    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body>
</w:document>"#,
        out.body
    );
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;
    let document_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>
</Relationships>"#;
    let core = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>Asten</dc:creator><dcterms:created xsi:type="dcterms:W3CDTF">{now}</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">{now}</dcterms:modified></cp:coreProperties>"#,
        escape_xml(title)
    );
    let app = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Asten</Application></Properties>"#;

    let parts: [(&str, String); 8] = [
        ("[Content_Types].xml", content_types.to_owned()),
        ("_rels/.rels", rels.to_owned()),
        ("docProps/core.xml", core),
        ("docProps/app.xml", app.to_owned()),
        ("word/document.xml", document_xml),
        ("word/_rels/document.xml.rels", document_rels.to_owned()),
        ("word/styles.xml", STYLES.to_owned()),
        ("word/numbering.xml", numbering(out.ordered_lists)),
    ];
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, body) in &parts {
            zip.start_file(*name, options)
                .map_err(|error| ApplicationError::Storage(error.to_string()))?;
            zip.write_all(body.as_bytes())
                .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        }
        zip.finish()
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
    }
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_domain::{DocumentId, DocumentKind, ProjectId, ScenePlacement};
    use serde_json::json;
    use std::io::Read;

    fn document(title: &str, content: Value) -> Document {
        let now = Utc::now();
        Document {
            id: DocumentId::new(),
            project_id: ProjectId::new(),
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: title.into(),
            content_json: content,
            position: 0,
            revision: 1,
            exclude_from_compile: false,
            scene: ScenePlacement::default(),
            head_seq: 0,
            chain_hash: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn read(bytes: &[u8], name: &str) -> String {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).unwrap();
        let mut text = String::new();
        zip.by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn marks_headings_lists_and_tracked_changes_are_written_as_word_writes_them() {
        let content = json!({"type":"doc","content":[
            {"type":"heading","attrs":{"level":1},"content":[{"type":"text","text":"Inner"}]},
            {"type":"paragraph","content":[
                {"type":"text","text":"She said "},
                {"type":"text","text":"never","marks":[{"type":"italic"}]},
                {"type":"text","text":" and "},
                {"type":"text","text":"loudly","marks":[{"type":"bold"},{"type":"underline"}]},
                {"type":"text","text":" new","marks":[{"type":"insertion"}]},
                {"type":"text","text":" old","marks":[{"type":"deletion"}]}
            ]},
            {"type":"sceneBreak"},
            {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Quoted"}]}]},
            {"type":"orderedList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"first"}]}]}]},
            {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"dot"}]}]}]}
        ]});
        let doc = document("One <&> Two", content);
        let bytes = to_docx("The Book", &[(0, &doc)]).unwrap();
        let xml = read(&bytes, "word/document.xml");
        assert!(xml.contains("<w:pStyle w:val=\"Title\"/>"));
        assert!(xml.contains("One &lt;&amp;&gt; Two"));
        assert!(
            xml.contains("<w:pStyle w:val=\"Heading2\"/>"),
            "an inner heading sits below the chapter title"
        );
        assert!(
            xml.contains("<w:r><w:rPr><w:i/></w:rPr><w:t xml:space=\"preserve\">never</w:t></w:r>")
        );
        assert!(xml.contains("<w:b/><w:u w:val=\"single\"/>"));
        assert!(xml.contains("<w:ins w:id=\"1\" w:author=\"Asten\""));
        assert!(xml.contains("<w:del w:id=\"2\" w:author=\"Asten\""));
        assert!(xml.contains("<w:delText xml:space=\"preserve\"> old</w:delText>"));
        assert!(xml.contains("<w:pStyle w:val=\"SceneBreak\"/>"));
        assert!(xml.contains("<w:pStyle w:val=\"Quote\"/>"));
        assert!(
            xml.contains("<w:numId w:val=\"11\"/>"),
            "numbered lists get their own numbering instance"
        );
        assert!(xml.contains("<w:numId w:val=\"1\"/>"));
        // Every part is well-formed XML.
        for name in [
            "[Content_Types].xml",
            "_rels/.rels",
            "docProps/core.xml",
            "word/document.xml",
            "word/styles.xml",
            "word/numbering.xml",
            "word/_rels/document.xml.rels",
        ] {
            let text = read(&bytes, name);
            let mut reader = quick_xml::Reader::from_str(&text);
            loop {
                match reader
                    .read_event()
                    .unwrap_or_else(|e| panic!("{name}: {e}"))
                {
                    quick_xml::events::Event::Eof => break,
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn what_is_exported_can_be_imported_again() {
        let content = json!({"type":"doc","content":[
            {"type":"paragraph","content":[
                {"type":"text","text":"Plain "},
                {"type":"text","text":"slanted","marks":[{"type":"italic"}]},
                {"type":"text","text":" kept","marks":[{"type":"insertion"}]}
            ]},
            {"type":"sceneBreak"},
            {"type":"paragraph","content":[{"type":"text","text":"After."}]}
        ]});
        let doc = document("Chapter A", content);
        let bytes = to_docx("T", &[(0, &doc)]).unwrap();
        let imported = crate::import::plan(&crate::import::ImportRequest {
            filename: "t.docx".into(),
            format: crate::import::ImportFormat::Docx,
            body: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes),
            encoding: Some("base64".into()),
            parent_id: None,
            kind: None,
            split: None,
        })
        .unwrap();
        let page = &imported.documents[0].content;
        let blocks = page["content"].as_array().unwrap();
        // Title, chapter heading, paragraph, scene break, paragraph.
        assert_eq!(blocks[0]["type"], "heading");
        assert_eq!(blocks[2]["content"][1]["marks"][0]["type"], "italic");
        assert_eq!(blocks[2]["content"][2]["marks"][0]["type"], "insertion");
        assert_eq!(blocks[3]["type"], "sceneBreak");
        assert_eq!(imported.insertions, 1);
    }
}
