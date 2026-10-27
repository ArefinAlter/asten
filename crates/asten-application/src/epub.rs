//! EPUB 3 for the compiled binder: one XHTML file per document in binder
//! order, a navigation document that keeps the binder's nesting, an EPUB 2
//! `toc.ncx` for older readers, a small stylesheet, and a package whose
//! `mimetype` entry comes first and uncompressed, as the container spec
//! requires. Every part is written as well-formed XML.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use asten_domain::{Document, document_plain_text};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::compile::escape_xml;
use crate::{ApplicationError, Result};

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#;

const STYLESHEET: &str = r#"body { margin: 0 auto; padding: 0 1.5em; line-height: 1.5; font-family: Georgia, "Times New Roman", serif; }
h1, h2, h3, h4, h5, h6 { font-weight: normal; line-height: 1.2; margin: 2em 0 1em; text-align: left; page-break-after: avoid; }
h1 { font-size: 1.6em; }
h2 { font-size: 1.35em; }
h3 { font-size: 1.15em; }
p { margin: 0; text-indent: 1.5em; text-align: left; }
h1 + p, h2 + p, h3 + p, h4 + p, h5 + p, h6 + p, hr + p, .scene-break + p, blockquote p, li p { text-indent: 0; }
.scene-break { text-indent: 0; text-align: center; margin: 1.5em 0; letter-spacing: 0.5em; }
hr { border: 0; border-top: 1px solid currentColor; width: 30%; margin: 1.5em auto; }
blockquote { margin: 1em 2em; }
pre { white-space: pre-wrap; font-size: 0.9em; }
nav ol { list-style: none; padding-left: 1em; }
"#;

struct Entry<'a> {
    depth: usize,
    document: &'a Document,
    id: String,
    href: String,
    title: String,
}

struct NavNode {
    index: usize,
    children: Vec<NavNode>,
}

/// The binder as an EPUB 3 container. `ordered` is the compiled documents in
/// binder order with their nesting depth.
pub(crate) fn to_epub(title: &str, ordered: &[(usize, &Document)]) -> Result<Vec<u8>> {
    let language = epub_language(ordered);
    let identifier = format!("urn:uuid:{}", Uuid::now_v7());
    let modified = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let entries: Vec<Entry> = ordered
        .iter()
        .enumerate()
        .map(|(index, (depth, document))| {
            let number = index + 1;
            let title = match document.title.trim() {
                "" => format!("Section {number}"),
                trimmed => trimmed.to_owned(),
            };
            Entry {
                depth: *depth,
                document,
                id: format!("doc-{number}"),
                href: format!("text/{number:03}-{}.xhtml", short_slug(&title)),
                title,
            }
        })
        .collect();
    let tree = nav_tree(&entries);

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        add(&mut zip, "mimetype", b"application/epub+zip", stored)?;
        add(
            &mut zip,
            "META-INF/container.xml",
            CONTAINER_XML.as_bytes(),
            deflated,
        )?;
        add(
            &mut zip,
            "OEBPS/package.opf",
            package_opf(title, &identifier, &language, &modified, &entries).as_bytes(),
            deflated,
        )?;
        add(
            &mut zip,
            "OEBPS/nav.xhtml",
            nav_xhtml(title, &language, &entries, &tree).as_bytes(),
            deflated,
        )?;
        add(
            &mut zip,
            "OEBPS/toc.ncx",
            toc_ncx(title, &identifier, &entries, &tree).as_bytes(),
            deflated,
        )?;
        add(&mut zip, "OEBPS/style.css", STYLESHEET.as_bytes(), deflated)?;
        for entry in &entries {
            add(
                &mut zip,
                &format!("OEBPS/{}", entry.href),
                document_xhtml(entry, &language).as_bytes(),
                deflated,
            )?;
        }
        zip.finish().map_err(storage)?;
    }
    Ok(cursor.into_inner())
}

fn add(
    zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>,
    name: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> Result<()> {
    zip.start_file(name, options).map_err(storage)?;
    zip.write_all(bytes).map_err(storage)
}

fn storage(error: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::Storage(error.to_string())
}

/// BCP 47 for `dc:language`: the language tag authors set on their scenes
/// when one dominates, else the dominant script as `und-<Script>`, else `und`.
fn epub_language(ordered: &[(usize, &Document)]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, document) in ordered {
        if let Some(tag) = document
            .scene
            .language
            .as_deref()
            .map(str::trim)
            .filter(|tag| looks_like_language_tag(tag))
        {
            *counts.entry(tag).or_default() += 1;
        }
    }
    if let Some((tag, _)) = counts.into_iter().max_by_key(|(_, count)| *count) {
        return tag.to_owned();
    }
    let text = ordered
        .iter()
        .map(|(_, document)| document_plain_text(&document.content_json))
        .collect::<Vec<_>>()
        .join(" ");
    match asten_perceive::detect_script(&text) {
        "Zyyy" => "und".to_owned(),
        script => format!("und-{script}"),
    }
}

fn looks_like_language_tag(tag: &str) -> bool {
    let mut parts = tag.split('-');
    let primary = parts.next().unwrap_or_default();
    (2..=3).contains(&primary.len())
        && primary.chars().all(|ch| ch.is_ascii_alphabetic())
        && parts.all(|part| {
            (1..=8).contains(&part.len()) && part.chars().all(|ch| ch.is_ascii_alphanumeric())
        })
}

fn short_slug(title: &str) -> String {
    let slug: String = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let mut slug = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    slug.truncate(40);
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "section".to_owned()
    } else {
        slug.to_owned()
    }
}

fn package_opf(
    title: &str,
    identifier: &str,
    language: &str,
    modified: &str,
    entries: &[Entry],
) -> String {
    let mut manifest = String::new();
    let mut spine = String::new();
    for entry in entries {
        manifest.push_str(&format!(
            "    <item id=\"{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
            entry.id,
            escape_xml(&entry.href)
        ));
        spine.push_str(&format!("    <itemref idref=\"{}\"/>\n", entry.id));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id" xml:lang="{language}">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">{identifier}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:language>{language}</dc:language>
    <meta property="dcterms:modified">{modified}</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="css" href="style.css" media-type="text/css"/>
{manifest}  </manifest>
  <spine toc="ncx">
{spine}  </spine>
</package>
"#,
        title = escape_xml(title),
        language = escape_xml(language),
    )
}

/// The binder's nesting rebuilt from the depth sequence. Depths only ever
/// step up by one, so a stack of open levels is enough; anything odd is
/// flattened to its own level rather than dropped.
fn nav_tree(entries: &[Entry]) -> Vec<NavNode> {
    let mut cursor = 0;
    build_nodes(entries, &mut cursor, 0)
}

fn build_nodes(entries: &[Entry], cursor: &mut usize, depth: usize) -> Vec<NavNode> {
    let mut nodes = Vec::new();
    while *cursor < entries.len() {
        let entry_depth = entries[*cursor].depth;
        if entry_depth < depth {
            break;
        }
        let index = *cursor;
        *cursor += 1;
        let children = build_nodes(entries, cursor, entry_depth.max(depth) + 1);
        nodes.push(NavNode { index, children });
    }
    nodes
}

fn nav_xhtml(title: &str, language: &str, entries: &[Entry], tree: &[NavNode]) -> String {
    let mut list = String::new();
    write_nav_list(tree, entries, &mut list, 2);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{language}" lang="{language}">
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>Contents</h1>
{list}  </nav>
</body>
</html>
"#,
        title = escape_xml(title),
        language = escape_xml(language),
    )
}

fn write_nav_list(nodes: &[NavNode], entries: &[Entry], out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{pad}<ol>\n"));
    for node in nodes {
        let entry = &entries[node.index];
        out.push_str(&format!(
            "{pad}  <li><a href=\"{}\">{}</a>",
            escape_xml(&entry.href),
            escape_xml(&entry.title)
        ));
        if node.children.is_empty() {
            out.push_str("</li>\n");
        } else {
            out.push('\n');
            write_nav_list(&node.children, entries, out, indent + 2);
            out.push_str(&format!("{pad}  </li>\n"));
        }
    }
    out.push_str(&format!("{pad}</ol>\n"));
}

fn toc_ncx(title: &str, identifier: &str, entries: &[Entry], tree: &[NavNode]) -> String {
    let depth = entries.iter().map(|entry| entry.depth).max().unwrap_or(0) + 1;
    let mut nav_map = String::new();
    let mut play_order = 0;
    write_nav_points(tree, entries, &mut nav_map, 2, &mut play_order);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{identifier}"/>
    <meta name="dtb:depth" content="{depth}"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{title}</text></docTitle>
  <navMap>
{nav_map}  </navMap>
</ncx>
"#,
        title = escape_xml(title),
    )
}

fn write_nav_points(
    nodes: &[NavNode],
    entries: &[Entry],
    out: &mut String,
    indent: usize,
    play_order: &mut usize,
) {
    let pad = "  ".repeat(indent);
    for node in nodes {
        *play_order += 1;
        let entry = &entries[node.index];
        out.push_str(&format!(
            "{pad}<navPoint id=\"np-{}\" playOrder=\"{}\">\n{pad}  <navLabel><text>{}</text></navLabel>\n{pad}  <content src=\"{}\"/>\n",
            node.index + 1,
            play_order,
            escape_xml(&entry.title),
            escape_xml(&entry.href)
        ));
        write_nav_points(&node.children, entries, out, indent + 1, play_order);
        out.push_str(&format!("{pad}</navPoint>\n"));
    }
}

fn document_xhtml(entry: &Entry, language: &str) -> String {
    let level = (entry.depth + 1).min(6);
    let mut body = format!("<h{level}>{}</h{level}>\n", escape_xml(&entry.title));
    match entry
        .document
        .content_json
        .get("content")
        .and_then(Value::as_array)
    {
        Some(blocks) => write_blocks(blocks, &mut body, entry.depth),
        None => {
            let text = document_plain_text(&entry.document.content_json);
            if !text.is_empty() {
                body.push_str(&format!("<p>{}</p>\n", escape_xml(&text)));
            }
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{language}" lang="{language}">
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <link rel="stylesheet" type="text/css" href="../style.css"/>
</head>
<body epub:type="bodymatter">
<section epub:type="chapter">
{body}</section>
</body>
</html>
"#,
        title = escape_xml(&entry.title),
        language = escape_xml(language),
    )
}

pub(crate) fn write_blocks(nodes: &[Value], out: &mut String, depth: usize) {
    for node in nodes {
        write_block(node, out, depth);
    }
}

fn write_block(node: &Value, out: &mut String, depth: usize) {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();
    let children = node
        .get("content")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    match node_type {
        "paragraph" => {
            out.push_str("<p>");
            write_inline_nodes(children, out);
            out.push_str("</p>\n");
        }
        "heading" => {
            let level = node
                .pointer("/attrs/level")
                .and_then(Value::as_u64)
                .unwrap_or(1) as usize;
            let level = (level + depth + 1).clamp(1, 6);
            out.push_str(&format!("<h{level}>"));
            write_inline_nodes(children, out);
            out.push_str(&format!("</h{level}>\n"));
        }
        "blockquote" => {
            out.push_str("<blockquote>\n");
            write_blocks(children, out, depth);
            out.push_str("</blockquote>\n");
        }
        "bulletList" => {
            out.push_str("<ul>\n");
            write_blocks(children, out, depth);
            out.push_str("</ul>\n");
        }
        "orderedList" => {
            out.push_str("<ol>\n");
            write_blocks(children, out, depth);
            out.push_str("</ol>\n");
        }
        "listItem" => {
            out.push_str("<li>");
            write_blocks(children, out, depth);
            out.push_str("</li>\n");
        }
        "codeBlock" => {
            out.push_str("<pre><code>");
            write_inline_nodes(children, out);
            out.push_str("</code></pre>\n");
        }
        "horizontalRule" => out.push_str("<hr/>\n"),
        "sceneBreak" => out.push_str("<p class=\"scene-break\">* * *</p>\n"),
        "text" | "hardBreak" => {
            out.push_str("<p>");
            write_inline(node, out);
            out.push_str("</p>\n");
        }
        _ => {
            if children.is_empty() {
                if let Some(text) = node.get("text").and_then(Value::as_str) {
                    out.push_str(&format!("<p>{}</p>\n", escape_xml(text)));
                }
            } else {
                write_blocks(children, out, depth);
            }
        }
    }
}

fn write_inline_nodes(nodes: &[Value], out: &mut String) {
    for node in nodes {
        write_inline(node, out);
    }
}

fn write_inline(node: &Value, out: &mut String) {
    if node.get("type").and_then(Value::as_str) == Some("hardBreak") {
        out.push_str("<br/>");
        return;
    }
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        let tags: Vec<&str> = node
            .get("marks")
            .and_then(Value::as_array)
            .map(|marks| {
                marks
                    .iter()
                    .filter_map(|mark| mark.get("type").and_then(Value::as_str))
                    .filter_map(mark_tag)
                    .collect()
            })
            .unwrap_or_default();
        for tag in &tags {
            out.push_str(&format!("<{tag}>"));
        }
        out.push_str(&escape_xml(text));
        for tag in tags.iter().rev() {
            out.push_str(&format!("</{tag}>"));
        }
        return;
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        write_inline_nodes(children, out);
    }
}

fn mark_tag(mark: &str) -> Option<&'static str> {
    match mark {
        "bold" | "strong" => Some("strong"),
        "italic" | "em" => Some("em"),
        "underline" => Some("u"),
        "strike" => Some("s"),
        "deletion" => Some("del"),
        "insertion" => Some("ins"),
        "code" => Some("code"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::io::{Cursor, Read};

    use asten_domain::{Document, DocumentId, DocumentKind, ProjectId, ScenePlacement};
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use chrono::{Duration, Utc};
    use quick_xml::{Reader, XmlVersion, events::Event};
    use serde_json::{Value, json};
    use zip::{CompressionMethod, ZipArchive};

    use crate::compile::{CompileFormat, compile_documents};

    fn document(
        project_id: ProjectId,
        parent_id: Option<DocumentId>,
        kind: DocumentKind,
        title: &str,
        position: i64,
        minute: i64,
        content: Value,
    ) -> Document {
        let created = Utc::now() + Duration::minutes(minute);
        Document {
            id: DocumentId::new(),
            project_id,
            parent_id,
            kind,
            title: title.into(),
            content_json: content,
            position,
            revision: 1,
            exclude_from_compile: false,
            scene: ScenePlacement::default(),
            head_seq: 0,
            chain_hash: String::new(),
            created_at: created,
            updated_at: created,
        }
    }

    fn paragraph(text: &str) -> Value {
        json!({ "type": "paragraph", "attrs": { "id": "p" }, "content": [{ "type": "text", "text": text }] })
    }

    /// A binder handed over in scrambled storage order: a manuscript with
    /// two chapters, scenes placed out of creation order, an excluded scene,
    /// a note, and a scene whose parent is not in the set.
    fn library() -> Vec<Document> {
        let project_id = ProjectId::new();
        let book = document(
            project_id,
            None,
            DocumentKind::Manuscript,
            "Book",
            0,
            0,
            json!({ "type": "doc", "content": [paragraph("A book opens.")] }),
        );
        let one = document(
            project_id,
            Some(book.id),
            DocumentKind::Chapter,
            "One",
            0,
            1,
            json!({ "type": "doc", "content": [] }),
        );
        let two = document(
            project_id,
            Some(book.id),
            DocumentKind::Chapter,
            "Two",
            1,
            2,
            json!({ "type": "doc", "content": [paragraph("Chapter two.")] }),
        );
        let scene_a = document(
            project_id,
            Some(one.id),
            DocumentKind::Scene,
            "A",
            1,
            3,
            json!({ "type": "doc", "content": [
                { "type": "heading", "attrs": { "id": "h", "level": 1 }, "content": [{ "type": "text", "text": "Dawn" }] },
                { "type": "bulletList", "attrs": { "id": "l" }, "content": [
                    { "type": "listItem", "attrs": { "id": "i" }, "content": [paragraph("one")] },
                    { "type": "listItem", "attrs": { "id": "j" }, "content": [paragraph("two")] }
                ] },
                { "type": "blockquote", "attrs": { "id": "q" }, "content": [paragraph("quoted")] }
            ] }),
        );
        let scene_b = document(
            project_id,
            Some(one.id),
            DocumentKind::Scene,
            "B & co <late>",
            0,
            4,
            json!({ "type": "doc", "content": [
                { "type": "paragraph", "attrs": { "id": "p1" }, "content": [
                    { "type": "text", "text": "She came " },
                    { "type": "text", "text": "late", "marks": [{ "type": "italic" }] },
                    { "type": "text", "text": " & left <early>." }
                ] },
                { "type": "sceneBreak", "attrs": { "id": "sb" } },
                { "type": "paragraph", "attrs": { "id": "p2" }, "content": [
                    { "type": "text", "text": "Bold", "marks": [{ "type": "bold" }, { "type": "underline" }] },
                    { "type": "hardBreak", "attrs": { "id": "br" } },
                    { "type": "text", "text": "after." }
                ] },
                { "type": "horizontalRule", "attrs": { "id": "hr" } }
            ] }),
        );
        let mut cut = document(
            project_id,
            Some(two.id),
            DocumentKind::Scene,
            "Cut",
            0,
            5,
            json!({ "type": "doc", "content": [paragraph("Never printed.")] }),
        );
        cut.exclude_from_compile = true;
        let notes = document(
            project_id,
            Some(book.id),
            DocumentKind::Note,
            "Notes",
            0,
            6,
            json!({ "type": "doc", "content": [paragraph("Private.")] }),
        );
        let orphan = document(
            project_id,
            Some(DocumentId::new()),
            DocumentKind::Scene,
            "Orphan",
            5,
            7,
            json!({ "text": "Legacy plain text." }),
        );
        vec![orphan, scene_a, two, cut, notes, scene_b, one, book]
    }

    fn read_entry(archive: &mut ZipArchive<Cursor<Vec<u8>>>, name: &str) -> String {
        let mut file = archive
            .by_name(name)
            .unwrap_or_else(|_| panic!("{name} missing from the epub"));
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        text
    }

    fn assert_well_formed(name: &str, xml: &str) {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => panic!("{name} is not well-formed: {error}"),
            }
        }
    }

    fn attribute(start: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
        start.attributes().flatten().find_map(|attribute| {
            (attribute.key.local_name().as_ref() == key).then(|| {
                attribute
                    .normalized_value(XmlVersion::Explicit1_0)
                    .unwrap()
                    .into_owned()
            })
        })
    }

    fn parse_package(opf: &str) -> (Vec<(String, String)>, Vec<String>) {
        let mut reader = Reader::from_str(opf);
        let mut manifest = Vec::new();
        let mut spine = Vec::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Eof => break,
                Event::Start(start) | Event::Empty(start) => match start.local_name().as_ref() {
                    "item" => manifest.push((
                        attribute(&start, "id").unwrap(),
                        attribute(&start, "href").unwrap(),
                    )),
                    "itemref" => spine.push(attribute(&start, "idref").unwrap()),
                    _ => {}
                },
                _ => {}
            }
        }
        (manifest, spine)
    }

    fn xml_title(xhtml: &str) -> String {
        let mut reader = Reader::from_str(xhtml);
        let mut in_title = false;
        let mut title = String::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Eof => break,
                Event::Start(start) if start.local_name().as_ref() == "title" => in_title = true,
                Event::End(end) if end.local_name().as_ref() == "title" => break,
                Event::Text(text) if in_title => title.push_str(&text.xml10_content()),
                Event::GeneralRef(reference) if in_title => {
                    let name = reference.into_inner();
                    title.push_str(quick_xml::escape::resolve_predefined_entity(&name).unwrap());
                }
                _ => {}
            }
        }
        title
    }

    #[test]
    fn epub_is_a_valid_container_in_binder_order() {
        let artifact = compile_documents("Dusk Harbor", &library(), &CompileFormat::Epub).unwrap();
        assert_eq!(artifact.filename, "dusk-harbor.epub");
        assert_eq!(artifact.mime, "application/epub+zip");
        assert_eq!(artifact.encoding, "base64");
        let bytes = STANDARD.decode(&artifact.body).unwrap();

        // The first local header: the mimetype entry, stored, with no extra field.
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        assert_eq!(
            u16::from_le_bytes([bytes[8], bytes[9]]),
            0,
            "mimetype must be stored"
        );
        assert_eq!(u16::from_le_bytes([bytes[26], bytes[27]]), 8);
        assert_eq!(
            u16::from_le_bytes([bytes[28], bytes[29]]),
            0,
            "no extra field"
        );
        assert_eq!(&bytes[30..38], b"mimetype");
        assert_eq!(&bytes[38..58], b"application/epub+zip");

        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        {
            let first = archive.by_index(0).unwrap();
            assert_eq!(first.name().unwrap(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
        }
        let names: HashSet<String> = archive
            .file_names()
            .map(|name| name.unwrap().into_owned())
            .collect();
        assert!(
            read_entry(&mut archive, "META-INF/container.xml")
                .contains("full-path=\"OEBPS/package.opf\"")
        );

        let opf = read_entry(&mut archive, "OEBPS/package.opf");
        let (manifest, spine) = parse_package(&opf);
        for (_, href) in &manifest {
            assert!(
                names.contains(&format!("OEBPS/{href}")),
                "{href} is in the manifest but not the zip"
            );
        }
        for id in ["nav", "ncx", "css"] {
            assert!(
                manifest.iter().any(|(item, _)| item == id),
                "{id} missing from manifest"
            );
        }
        assert!(opf.contains("properties=\"nav\""));
        assert!(opf.contains("<dc:identifier id=\"pub-id\">urn:uuid:"));
        assert!(opf.contains("<dc:title>Dusk Harbor</dc:title>"));
        assert!(opf.contains("<dc:language>und-Latn</dc:language>"));
        assert!(opf.contains("<meta property=\"dcterms:modified\">"));

        let hrefs: Vec<String> = spine
            .iter()
            .map(|idref| {
                manifest
                    .iter()
                    .find(|(id, _)| id == idref)
                    .unwrap_or_else(|| panic!("spine item {idref} is not in the manifest"))
                    .1
                    .clone()
            })
            .collect();
        let titles: Vec<String> = hrefs
            .iter()
            .map(|href| xml_title(&read_entry(&mut archive, &format!("OEBPS/{href}"))))
            .collect();
        assert_eq!(
            titles,
            ["Book", "One", "B & co <late>", "A", "Two", "Orphan"]
        );

        for name in &names {
            if name.ends_with(".xhtml")
                || name.ends_with(".opf")
                || name.ends_with(".ncx")
                || name.ends_with(".xml")
            {
                let xml = read_entry(&mut archive, name);
                assert_well_formed(name, &xml);
                assert!(
                    !xml.contains("Never printed"),
                    "{name} leaks an excluded scene"
                );
                assert!(!xml.contains("Private."), "{name} leaks a note");
            }
        }

        let nav = read_entry(&mut archive, "OEBPS/nav.xhtml");
        assert!(nav.contains("epub:type=\"toc\""));
        assert_eq!(
            nav.matches("<ol>").count(),
            3,
            "book > chapter > scene nests three lists"
        );
        assert!(nav.contains("B &amp; co &lt;late&gt;"));
        assert!(!nav.contains(">Cut<") && !nav.contains(">Notes<"));

        let ncx = read_entry(&mut archive, "OEBPS/toc.ncx");
        assert!(ncx.contains("<meta name=\"dtb:depth\" content=\"3\"/>"));
        assert!(ncx.contains("playOrder=\"6\""));
        assert!(!ncx.contains("playOrder=\"7\""));

        let scene_b = read_entry(&mut archive, &format!("OEBPS/{}", hrefs[2]));
        assert!(scene_b.contains("<h3>B &amp; co &lt;late&gt;</h3>"));
        assert!(scene_b.contains("She came <em>late</em> &amp; left &lt;early&gt;."));
        assert!(scene_b.contains("<p class=\"scene-break\">* * *</p>"));
        assert!(scene_b.contains("<strong><u>Bold</u></strong><br/>after."));
        assert!(scene_b.contains("<hr/>"));
        let scene_a = read_entry(&mut archive, &format!("OEBPS/{}", hrefs[3]));
        assert!(
            scene_a.contains("<h4>Dawn</h4>"),
            "headings sit below the scene title"
        );
        assert!(scene_a.contains("<ul>\n<li><p>one</p>\n</li>"));
        assert!(scene_a.contains("<blockquote>\n<p>quoted</p>\n</blockquote>"));
        let orphan = read_entry(&mut archive, &format!("OEBPS/{}", hrefs[5]));
        assert!(orphan.contains("<h1>Orphan</h1>\n<p>Legacy plain text.</p>"));
    }

    #[test]
    fn epub_language_follows_scene_tags_then_script() {
        let project_id = ProjectId::new();
        let mut tagged = document(
            project_id,
            None,
            DocumentKind::Scene,
            "Un",
            0,
            0,
            json!({ "type": "doc", "content": [paragraph("Bonjour.")] }),
        );
        tagged.scene.language = Some("fr".into());
        let mut labelled = document(
            project_id,
            None,
            DocumentKind::Scene,
            "Deux",
            1,
            1,
            json!({ "type": "doc", "content": [paragraph("Salut.")] }),
        );
        labelled.scene.language = Some("French prose".into());
        let artifact =
            compile_documents("Deux", &[tagged, labelled], &CompileFormat::Epub).unwrap();
        let bytes = STANDARD.decode(&artifact.body).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert!(
            read_entry(&mut archive, "OEBPS/package.opf").contains("<dc:language>fr</dc:language>")
        );

        let cyrillic = document(
            project_id,
            None,
            DocumentKind::Scene,
            "Раз",
            0,
            0,
            json!({ "type": "doc", "content": [paragraph("Она пришла поздно.")] }),
        );
        let artifact = compile_documents("Раз", &[cyrillic], &CompileFormat::Epub).unwrap();
        let bytes = STANDARD.decode(&artifact.body).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert!(
            read_entry(&mut archive, "OEBPS/package.opf")
                .contains("<dc:language>und-Cyrl</dc:language>")
        );
        assert_eq!(artifact.filename, "manuscript.epub");
    }

    #[test]
    fn other_formats_follow_binder_order_too() {
        let artifact =
            compile_documents("Dusk Harbor", &library(), &CompileFormat::Markdown).unwrap();
        let headings: Vec<&str> = artifact
            .body
            .lines()
            .filter_map(|line| line.strip_prefix("## "))
            .collect();
        assert_eq!(
            headings,
            ["Book", "One", "B & co <late>", "A", "Two", "Orphan"]
        );
    }
}
