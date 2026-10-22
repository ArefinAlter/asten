//! Final Draft's FDX: a plain XML script in which every paragraph carries its
//! element type. Export reads the types off the manuscript with the shared
//! screenplay classifier and drops Fountain's forcing marks, since the type
//! travels explicitly; import writes the types back as Fountain conventions,
//! adding a forcing mark only where the bare text would be misread, so a
//! script round-trips through either format.

use asten_domain::{
    Document, ScreenplayElement, classify_screenplay, empty_prosemirror_doc,
    force_screenplay_marks, is_slugline,
};
use quick_xml::{
    Reader, XmlVersion, escape::resolve_predefined_entity, events::BytesStart, events::Event,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::compile::escape_xml;
use crate::{ApplicationError, Result};

/// Styles as Final Draft spells them, in the order they are written.
const STYLES: [(&str, &str); 4] = [
    ("Bold", "bold"),
    ("Italic", "italic"),
    ("Underline", "underline"),
    ("Strikeout", "strike"),
];

#[derive(Clone, Debug, Default, PartialEq)]
struct Run {
    text: String,
    styles: Vec<&'static str>,
}

struct Paragraph {
    kind: &'static str,
    centered: bool,
    runs: Vec<Run>,
}

// ───────────────────────── export ─────────────────────────

pub(crate) fn to_fdx(title: &str, documents: &[&Document]) -> String {
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    for document in documents {
        let heading = document.title.trim();
        if !heading.is_empty() {
            paragraphs.push(Paragraph {
                kind: title_element(heading).fdx_type(),
                centered: false,
                runs: vec![Run {
                    text: heading.to_owned(),
                    styles: Vec::new(),
                }],
            });
        }
        let empty = Vec::new();
        let blocks = document
            .content_json
            .get("content")
            .and_then(Value::as_array)
            .unwrap_or(&empty);
        let mut index = 0;
        while index < blocks.len() {
            if node_type(&blocks[index]) == "paragraph" {
                let end = (index..blocks.len())
                    .find(|&at| node_type(&blocks[at]) != "paragraph")
                    .unwrap_or(blocks.len());
                let runs: Vec<Vec<Run>> = blocks[index..end]
                    .iter()
                    .map(|block| trim_runs(inline_runs(block)))
                    .filter(|runs| !plain_text(runs).is_empty())
                    .collect();
                let texts: Vec<String> = runs.iter().map(|runs| plain_text(runs)).collect();
                let kinds =
                    classify_screenplay(&texts.iter().map(String::as_str).collect::<Vec<_>>());
                for (runs, kind) in runs.into_iter().zip(kinds) {
                    paragraphs.push(Paragraph {
                        kind: kind.fdx_type(),
                        centered: false,
                        runs: strip_forcing_mark(runs, kind),
                    });
                }
                index = end;
                continue;
            }
            match node_type(&blocks[index]) {
                "heading" => {
                    let runs = trim_runs(inline_runs(&blocks[index]));
                    let text = plain_text(&runs);
                    if !text.is_empty() {
                        paragraphs.push(Paragraph {
                            kind: title_element(&text).fdx_type(),
                            centered: false,
                            runs,
                        });
                    }
                }
                "horizontalRule" | "sceneBreak" => paragraphs.push(Paragraph {
                    kind: ScreenplayElement::General.fdx_type(),
                    centered: true,
                    runs: vec![Run {
                        text: "* * *".into(),
                        styles: Vec::new(),
                    }],
                }),
                _ => {
                    let mut collected = Vec::new();
                    collect_text_blocks(&blocks[index], &mut collected);
                    for runs in collected {
                        paragraphs.push(Paragraph {
                            kind: ScreenplayElement::General.fdx_type(),
                            centered: false,
                            runs,
                        });
                    }
                }
            }
            index += 1;
        }
    }
    render(title, &paragraphs)
}

/// A document or heading title is a scene heading only when it reads as a
/// slugline; anything else is a general paragraph.
fn title_element(text: &str) -> ScreenplayElement {
    if is_slugline(text) {
        ScreenplayElement::SceneHeading
    } else {
        ScreenplayElement::General
    }
}

fn node_type(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or_default()
}

/// Inline content as styled runs; a hard break is a newline inside the paragraph.
fn inline_runs(node: &Value) -> Vec<Run> {
    let mut runs = Vec::new();
    collect_inline_runs(node, &mut runs);
    runs
}

fn collect_inline_runs(node: &Value, runs: &mut Vec<Run>) {
    if node_type(node) == "hardBreak" {
        runs.push(Run {
            text: "\n".into(),
            styles: Vec::new(),
        });
        return;
    }
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        let marks: Vec<&str> = node
            .get("marks")
            .and_then(Value::as_array)
            .map(|marks| {
                marks
                    .iter()
                    .filter_map(|mark| mark.get("type").and_then(Value::as_str))
                    .collect()
            })
            .unwrap_or_default();
        let styles: Vec<&'static str> = STYLES
            .iter()
            .filter(|(_, mark)| {
                marks.contains(mark)
                    || (*mark == "bold" && marks.contains(&"strong"))
                    || (*mark == "italic" && marks.contains(&"em"))
            })
            .map(|(style, _)| *style)
            .collect();
        runs.push(Run {
            text: text.to_owned(),
            styles,
        });
        return;
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            collect_inline_runs(child, runs);
        }
    }
}

/// Every paragraph-like block inside a container (quote, list, code block).
fn collect_text_blocks(node: &Value, out: &mut Vec<Vec<Run>>) {
    match node_type(node) {
        "paragraph" | "heading" | "codeBlock" => {
            let runs = trim_runs(inline_runs(node));
            if !plain_text(&runs).is_empty() {
                out.push(runs);
            }
        }
        _ => {
            if node.get("text").is_some() {
                let runs = trim_runs(inline_runs(node));
                if !plain_text(&runs).is_empty() {
                    out.push(runs);
                }
            } else if let Some(children) = node.get("content").and_then(Value::as_array) {
                for child in children {
                    collect_text_blocks(child, out);
                }
            }
        }
    }
}

fn plain_text(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

/// Drops empty runs and the whitespace around the paragraph, keeping styles.
fn trim_runs(runs: Vec<Run>) -> Vec<Run> {
    let plain = plain_text(&runs);
    let start = plain.len() - plain.trim_start().len();
    let end = plain.trim_end().len();
    slice_runs(runs, start, end)
}

/// The runs covering byte range `start..end` of their concatenated text.
fn slice_runs(runs: Vec<Run>, start: usize, end: usize) -> Vec<Run> {
    let mut offset = 0;
    let mut sliced = Vec::new();
    for run in runs {
        let run_start = offset;
        let run_end = offset + run.text.len();
        offset = run_end;
        let keep_start = start.clamp(run_start, run_end) - run_start;
        let keep_end = end.clamp(run_start, run_end) - run_start;
        if keep_end > keep_start {
            sliced.push(Run {
                text: run.text[keep_start..keep_end].to_owned(),
                styles: run.styles,
            });
        }
    }
    sliced
}

/// Removes the Fountain mark that forced this element, and a dual-dialogue
/// caret, since FDX carries the type on the paragraph.
fn strip_forcing_mark(runs: Vec<Run>, kind: ScreenplayElement) -> Vec<Run> {
    let plain = plain_text(&runs);
    let mut start = 0;
    if let Some(mark) = kind.forcing_mark() {
        let forced = plain.starts_with(mark)
            && !(kind == ScreenplayElement::SceneHeading && plain.starts_with(".."));
        if forced {
            start = mark.len_utf8();
            start += plain[start..].len() - plain[start..].trim_start().len();
        }
    }
    let mut end = plain.len();
    if kind == ScreenplayElement::Character {
        end = plain.trim_end_matches('^').trim_end().len().max(start);
    }
    slice_runs(runs, start, end)
}

fn render(title: &str, paragraphs: &[Paragraph]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"1\">\n  <Content>\n",
    );
    for paragraph in paragraphs {
        write_paragraph(&mut out, paragraph, "    ");
    }
    out.push_str("  </Content>\n  <TitlePage>\n    <Content>\n");
    write_paragraph(
        &mut out,
        &Paragraph {
            kind: ScreenplayElement::General.fdx_type(),
            centered: true,
            runs: vec![Run {
                text: title.trim().to_owned(),
                styles: Vec::new(),
            }],
        },
        "      ",
    );
    out.push_str("    </Content>\n  </TitlePage>\n</FinalDraft>\n");
    out
}

fn write_paragraph(out: &mut String, paragraph: &Paragraph, indent: &str) {
    out.push_str(indent);
    out.push_str("<Paragraph Type=\"");
    out.push_str(paragraph.kind);
    out.push('"');
    if paragraph.centered {
        out.push_str(" Alignment=\"Center\"");
    }
    out.push_str(">\n");
    if paragraph.runs.is_empty() {
        out.push_str(indent);
        out.push_str("  <Text></Text>\n");
    }
    for run in &paragraph.runs {
        out.push_str(indent);
        out.push_str("  <Text");
        if !run.styles.is_empty() {
            out.push_str(" Style=\"");
            out.push_str(&run.styles.join("+"));
            out.push('"');
        }
        out.push('>');
        out.push_str(&escape_xml(&run.text));
        out.push_str("</Text>\n");
    }
    out.push_str(indent);
    out.push_str("</Paragraph>\n");
}

// ───────────────────────── import ─────────────────────────

struct OpenParagraph {
    kind: String,
    runs: Vec<Run>,
    depth: usize,
}

enum Block {
    Element(ScreenplayElement, Vec<Run>),
    Rule,
}

/// A Final Draft document as ProseMirror paragraphs. Only the script body is
/// read: the title page, script notes and settings are skipped. Character
/// names come back uppercase and parentheticals in parentheses, the way the
/// Fountain importer leaves them.
pub(crate) fn fdx_to_prosemirror(xml: &str) -> Result<Value> {
    let xml = xml.trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut root_seen = false;
    let mut current: Option<OpenParagraph> = None;
    let mut run: Option<Run> = None;
    let mut blocks: Vec<Block> = Vec::new();
    loop {
        let event = reader.read_event().map_err(|error| {
            ApplicationError::Validation(format!("not a Final Draft document: {error}"))
        })?;
        match event {
            Event::Start(start) => {
                let name = start.local_name().as_ref().to_owned();
                check_root(&name, &mut root_seen)?;
                open_element(&name, &start, &stack, &mut current, &mut run)?;
                stack.push(name);
            }
            Event::Empty(start) => {
                let name = start.local_name().as_ref().to_owned();
                check_root(&name, &mut root_seen)?;
            }
            Event::End(_) => {
                if let Some(name) = stack.pop() {
                    close_element(&name, stack.len(), &mut current, &mut run, &mut blocks);
                }
            }
            Event::Text(text) => {
                if let Some(run) = run.as_mut() {
                    run.text.push_str(&text.xml10_content());
                }
            }
            Event::CData(data) => {
                if let Some(run) = run.as_mut() {
                    run.text.push_str(&data.into_inner());
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(run) = run.as_mut() {
                    if reference.is_char_ref() {
                        if let Ok(Some(ch)) = reference.resolve_char_ref() {
                            run.text.push(ch);
                        }
                    } else {
                        let name = reference.into_inner();
                        match resolve_predefined_entity(&name) {
                            Some(text) => run.text.push_str(text),
                            None => run.text.push_str(&format!("&{name};")),
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if let Some(open) = stack.last() {
        return Err(ApplicationError::Validation(format!(
            "not a Final Draft document: the file ends inside <{open}>"
        )));
    }
    if !root_seen {
        return Err(ApplicationError::Validation(
            "not a Final Draft document: no FinalDraft element".into(),
        ));
    }
    Ok(blocks_to_prosemirror(blocks))
}

fn check_root(name: &str, root_seen: &mut bool) -> Result<()> {
    if *root_seen {
        return Ok(());
    }
    if name != "FinalDraft" {
        return Err(ApplicationError::Validation(format!(
            "not a Final Draft document: the root element is <{name}>, not <FinalDraft>"
        )));
    }
    *root_seen = true;
    Ok(())
}

fn attribute(start: &BytesStart, key: &str) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| {
            ApplicationError::Validation(format!("not a Final Draft document: {error}"))
        })?;
        if attribute.key.local_name().as_ref() == key {
            let value = attribute
                .normalized_value(XmlVersion::Explicit1_0)
                .map_err(|error| {
                    ApplicationError::Validation(format!("not a Final Draft document: {error}"))
                })?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

fn open_element(
    name: &str,
    start: &BytesStart,
    stack: &[String],
    current: &mut Option<OpenParagraph>,
    run: &mut Option<Run>,
) -> Result<()> {
    match name {
        "Paragraph" => {
            let skipped = stack
                .iter()
                .any(|open| open == "TitlePage" || open == "ScriptNote");
            if current.is_some() || skipped {
                return Ok(());
            }
            *current = Some(OpenParagraph {
                kind: attribute(start, "Type")?.unwrap_or_else(|| "General".to_owned()),
                runs: Vec::new(),
                depth: stack.len(),
            });
        }
        "Text" => {
            let direct_child = current
                .as_ref()
                .is_some_and(|open| stack.len() == open.depth + 1);
            if direct_child {
                let styles = attribute(start, "Style")?
                    .map(|value| {
                        value
                            .split('+')
                            .filter_map(|part| {
                                STYLES
                                    .iter()
                                    .find(|(style, _)| style.eq_ignore_ascii_case(part.trim()))
                                    .map(|(style, _)| *style)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                *run = Some(Run {
                    text: String::new(),
                    styles,
                });
            }
        }
        _ => {}
    }
    Ok(())
}

fn close_element(
    name: &str,
    depth: usize,
    current: &mut Option<OpenParagraph>,
    run: &mut Option<Run>,
    blocks: &mut Vec<Block>,
) {
    match name {
        "Text" => {
            if let Some(finished) = run.take() {
                if let Some(open) = current.as_mut() {
                    open.runs.push(finished);
                }
            }
        }
        "Paragraph" if current.as_ref().is_some_and(|open| open.depth == depth) => {
            if let Some(open) = current.take() {
                finish_paragraph(open, blocks);
            }
        }
        _ => {}
    }
}

fn finish_paragraph(open: OpenParagraph, blocks: &mut Vec<Block>) {
    let mut runs = trim_runs(open.runs);
    let plain = plain_text(&runs);
    if plain.is_empty() {
        return;
    }
    if plain == "* * *" || plain == "***" || plain == "---" {
        blocks.push(Block::Rule);
        return;
    }
    let element = ScreenplayElement::from_fdx_type(&open.kind);
    match element {
        ScreenplayElement::Character => {
            for run in &mut runs {
                run.text = run.text.to_uppercase();
            }
        }
        ScreenplayElement::Parenthetical => {
            if !plain.starts_with('(') {
                if let Some(first) = runs.first_mut() {
                    first.text.insert(0, '(');
                }
            }
            if !plain.ends_with(')') {
                if let Some(last) = runs.last_mut() {
                    last.text.push(')');
                }
            }
        }
        _ => {}
    }
    blocks.push(Block::Element(element, runs));
}

fn blocks_to_prosemirror(blocks: Vec<Block>) -> Value {
    let mut content: Vec<Value> = Vec::new();
    let mut index = 0;
    while index < blocks.len() {
        if matches!(blocks[index], Block::Rule) {
            content.push(json!({
                "type": "horizontalRule",
                "attrs": { "id": Uuid::now_v7().to_string() }
            }));
            index += 1;
            continue;
        }
        let end = (index..blocks.len())
            .find(|&at| matches!(blocks[at], Block::Rule))
            .unwrap_or(blocks.len());
        let elements: Vec<(ScreenplayElement, String)> = blocks[index..end]
            .iter()
            .filter_map(|block| match block {
                Block::Element(element, runs) => Some((*element, plain_text(runs))),
                Block::Rule => None,
            })
            .collect();
        let forced = force_screenplay_marks(&elements);
        for (at, block) in blocks[index..end].iter().enumerate() {
            let Block::Element(_, runs) = block else {
                continue;
            };
            let mut runs = runs.clone();
            let plain = &elements[at].1;
            if forced[at].len() > plain.len() {
                let prefix = forced[at][..forced[at].len() - plain.len()].to_owned();
                if let Some(first) = runs.first_mut() {
                    first.text.insert_str(0, &prefix);
                }
            }
            content.push(paragraph_node(&runs));
        }
        index = end;
    }
    if content.is_empty() {
        return empty_prosemirror_doc();
    }
    json!({ "type": "doc", "content": content })
}

fn paragraph_node(runs: &[Run]) -> Value {
    let content: Vec<Value> = runs
        .iter()
        .filter(|run| !run.text.is_empty())
        .map(|run| {
            let marks: Vec<Value> = run
                .styles
                .iter()
                .filter_map(|style| {
                    STYLES
                        .iter()
                        .find(|(name, _)| name == style)
                        .map(|(_, mark)| json!({ "type": mark }))
                })
                .collect();
            if marks.is_empty() {
                json!({ "type": "text", "text": run.text })
            } else {
                json!({ "type": "text", "text": run.text, "marks": marks })
            }
        })
        .collect();
    json!({
        "type": "paragraph",
        "attrs": { "id": Uuid::now_v7().to_string() },
        "content": content
    })
}

#[cfg(test)]
mod tests {
    use asten_domain::{
        Document, DocumentId, DocumentKind, ProjectId, ScenePlacement, fountain_to_prosemirror,
    };
    use chrono::Utc;
    use quick_xml::{Reader, XmlVersion, events::Event};
    use serde_json::{Value, json};

    use super::{fdx_to_prosemirror, to_fdx};
    use crate::compile::{CompileFormat, compile_documents};

    fn document(title: &str, content: Value) -> Document {
        let now = Utc::now();
        Document {
            id: DocumentId::new(),
            project_id: ProjectId::new(),
            parent_id: None,
            kind: DocumentKind::Scene,
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

    fn paragraph(text: &str) -> Value {
        json!({ "type": "paragraph", "attrs": { "id": "p" }, "content": [{ "type": "text", "text": text }] })
    }

    /// `(Type, text)` for every paragraph in the script body, title page excluded.
    fn script_paragraphs(xml: &str) -> Vec<(String, String)> {
        let mut reader = Reader::from_str(xml);
        let mut stack: Vec<String> = Vec::new();
        let mut paragraphs = Vec::new();
        let mut kind: Option<String> = None;
        let mut text = String::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Eof => break,
                Event::Start(start) => {
                    let name = start.local_name().as_ref().to_owned();
                    if name == "Paragraph" && !stack.iter().any(|open| open == "TitlePage") {
                        kind = start.attributes().flatten().find_map(|attribute| {
                            (attribute.key.local_name().as_ref() == "Type").then(|| {
                                attribute
                                    .normalized_value(XmlVersion::Explicit1_0)
                                    .unwrap()
                                    .into_owned()
                            })
                        });
                        text.clear();
                    }
                    stack.push(name);
                }
                Event::End(end) => {
                    stack.pop();
                    if end.local_name().as_ref() == "Paragraph" {
                        if let Some(kind) = kind.take() {
                            paragraphs.push((kind, text.clone()));
                        }
                    }
                }
                Event::Text(run)
                    if kind.is_some() && stack.last().map(String::as_str) == Some("Text") =>
                {
                    text.push_str(&run.xml10_content());
                }
                Event::GeneralRef(reference) if kind.is_some() => {
                    let name = reference.into_inner();
                    text.push_str(quick_xml::escape::resolve_predefined_entity(&name).unwrap());
                }
                _ => {}
            }
        }
        paragraphs
    }

    fn texts(doc: &Value) -> Vec<(String, String)> {
        doc["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| {
                let text = node["content"]
                    .as_array()
                    .map(|runs| {
                        runs.iter()
                            .filter_map(|run| run["text"].as_str())
                            .collect::<String>()
                    })
                    .unwrap_or_default();
                (node["type"].as_str().unwrap().to_owned(), text)
            })
            .collect()
    }

    #[test]
    fn fdx_export_types_every_paragraph_like_fountain_would() {
        let scene = document(
            "INT. KITCHEN - NIGHT",
            json!({ "type": "doc", "content": [
                { "type": "paragraph", "attrs": { "id": "a" }, "content": [
                    { "type": "text", "text": "Mira " },
                    { "type": "text", "text": "waits", "marks": [{ "type": "italic" }, { "type": "bold" }] },
                    { "type": "text", "text": " by the stove & hums <softly>." }
                ] },
                paragraph("MIRA ^"),
                paragraph("(quietly)"),
                paragraph("I knew you'd come."),
                paragraph("!LATER"),
                paragraph("CUT TO:"),
                { "type": "sceneBreak", "attrs": { "id": "sb" } },
                { "type": "heading", "attrs": { "id": "h", "level": 2 }, "content": [{ "type": "text", "text": "Act Two" }] },
                { "type": "bulletList", "attrs": { "id": "l" }, "content": [
                    { "type": "listItem", "attrs": { "id": "i" }, "content": [paragraph("a prop")] }
                ] },
                paragraph(".CONTINUOUS"),
                paragraph("@tom"),
                paragraph("Hi.")
            ] }),
        );
        let later = document(
            "Scene 2",
            json!({ "type": "doc", "content": [paragraph("Rain.")] }),
        );
        let artifact =
            compile_documents("Dusk Harbor", &[scene, later], &CompileFormat::Fdx).unwrap();
        assert_eq!(artifact.filename, "dusk-harbor.fdx");
        assert_eq!(artifact.mime, "application/xml");
        assert_eq!(artifact.encoding, "utf-8");
        let xml = &artifact.body;
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"1\">"));
        assert!(xml.contains("<Text Style=\"Bold+Italic\">waits</Text>"));
        assert!(xml.contains("by the stove &amp; hums &lt;softly&gt;."));
        assert!(xml.contains("<TitlePage>\n    <Content>\n      <Paragraph Type=\"General\" Alignment=\"Center\">\n        <Text>Dusk Harbor</Text>"));
        assert!(xml.contains(
            "    <Paragraph Type=\"General\" Alignment=\"Center\">\n      <Text>* * *</Text>"
        ));
        let expected: Vec<(String, String)> = [
            ("Scene Heading", "INT. KITCHEN - NIGHT"),
            ("Action", "Mira waits by the stove & hums <softly>."),
            ("Character", "MIRA"),
            ("Parenthetical", "(quietly)"),
            ("Dialogue", "I knew you'd come."),
            ("Action", "LATER"),
            ("Transition", "CUT TO:"),
            ("General", "* * *"),
            ("General", "Act Two"),
            ("General", "a prop"),
            ("Scene Heading", "CONTINUOUS"),
            ("Character", "tom"),
            ("Dialogue", "Hi."),
            ("General", "Scene 2"),
            ("Action", "Rain."),
        ]
        .iter()
        .map(|(kind, text)| ((*kind).to_owned(), (*text).to_owned()))
        .collect();
        assert_eq!(script_paragraphs(xml), expected);
    }

    #[test]
    fn fountain_to_fdx_to_import_is_stable() {
        let fountain = "Title: Dusk\n\nFADE IN:\n\nINT. KITCHEN - NIGHT\n\nMira waits by the stove.\nThe kettle ticks.\n\nMIRA\n(quietly)\nI knew you'd come.\nSit down.\n(beat)\nPlease.\n\nBANG.\n\nLATER\n\nThe door slams.\n\n> FADE OUT.\n\n===\n\nEXT. HARBOUR - DAWN\n\nJO\nMorning.\n";
        let original = fountain_to_prosemirror(fountain);
        let first = document("INT. KITCHEN - NIGHT", original.clone());
        let imported = fdx_to_prosemirror(&to_fdx("Dusk", &[&first])).unwrap();
        let mut expected = vec![("paragraph".to_owned(), "INT. KITCHEN - NIGHT".to_owned())];
        expected.extend(texts(&original));
        assert_eq!(texts(&imported), expected);
        assert!(texts(&original).iter().any(|(_, text)| text == "!LATER"));
        assert!(
            texts(&original)
                .iter()
                .any(|(_, text)| text == "> FADE OUT." || text == ">FADE OUT.")
        );

        // A second trip changes nothing but the repeated title paragraph.
        let second = document("INT. KITCHEN - NIGHT", imported.clone());
        let again = fdx_to_prosemirror(&to_fdx("Dusk", &[&second])).unwrap();
        let mut expected = vec![("paragraph".to_owned(), "INT. KITCHEN - NIGHT".to_owned())];
        expected.extend(texts(&imported));
        assert_eq!(texts(&again), expected);
        assert_eq!(
            script_paragraphs(&to_fdx("Dusk", &[&second]))[1..],
            script_paragraphs(&to_fdx("Dusk", &[&first]))[..]
        );
    }

    #[test]
    fn fdx_import_reads_a_final_draft_file() {
        let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>\n<FinalDraft DocumentType=\"Script\" Template=\"No\" Version=\"5\">\n  <Content>\n    <Paragraph Type=\"Scene Heading\" Number=\"1\">\n      <SceneProperties Length=\"1/8\" Page=\"1\"/>\n      <Text>INT. KITCHEN &#8211; NIGHT</Text>\n    </Paragraph>\n    <Paragraph Type=\"Action\">\n      <Text>Mira </Text><Text Style=\"Bold+Underline\">waits</Text><Text> &amp; hums.</Text>\n      <ScriptNote><Paragraph><Text>tighten this</Text></Paragraph></ScriptNote>\n    </Paragraph>\n    <Paragraph Type=\"Action\">\n      <Text></Text>\n    </Paragraph>\n    <Paragraph Type=\"Character\">\n      <Text>mira (cont'd)</Text>\n    </Paragraph>\n    <Paragraph Type=\"Parenthetical\">\n      <Text>beat</Text>\n    </Paragraph>\n    <Paragraph Type=\"Dialogue\">\n      <Text>I knew you<![CDATA[']]>d come.</Text>\n    </Paragraph>\n    <Paragraph Type=\"Action\">\n      <Text>LATER</Text>\n    </Paragraph>\n    <Paragraph Type=\"Action\">\n      <Text>The door slams.</Text>\n    </Paragraph>\n    <Paragraph Type=\"General\" Alignment=\"Center\">\n      <Text>* * *</Text>\n    </Paragraph>\n    <Paragraph Type=\"Scene Heading\">\n      <Text>CONTINUOUS</Text>\n    </Paragraph>\n    <Paragraph Type=\"Transition\">\n      <Text>FADE OUT.</Text>\n    </Paragraph>\n    <DualDialogue>\n      <Paragraph Type=\"Character\"><Text>JO</Text></Paragraph>\n      <Paragraph Type=\"Dialogue\"><Text>Morning.</Text></Paragraph>\n    </DualDialogue>\n  </Content>\n  <TitlePage>\n    <Content>\n      <Paragraph Type=\"General\" Alignment=\"Center\"><Text>Dusk</Text></Paragraph>\n    </Content>\n  </TitlePage>\n  <ElementSettings Type=\"Action\"><FontSpec Font=\"Courier\"/></ElementSettings>\n</FinalDraft>\n";
        let doc = fdx_to_prosemirror(xml).unwrap();
        let expected: Vec<(String, String)> = [
            ("paragraph", "INT. KITCHEN \u{2013} NIGHT"),
            ("paragraph", "Mira waits & hums."),
            ("paragraph", "MIRA (CONT'D)"),
            ("paragraph", "(beat)"),
            ("paragraph", "I knew you'd come."),
            ("paragraph", "!LATER"),
            ("paragraph", "The door slams."),
            ("horizontalRule", ""),
            ("paragraph", ".CONTINUOUS"),
            ("paragraph", "> FADE OUT."),
            ("paragraph", "JO"),
            ("paragraph", "Morning."),
        ]
        .iter()
        .map(|(kind, text)| ((*kind).to_owned(), (*text).to_owned()))
        .collect();
        assert_eq!(texts(&doc), expected);
        assert_eq!(
            doc["content"][1]["content"][1],
            json!({ "type": "text", "text": "waits", "marks": [{ "type": "bold" }, { "type": "underline" }] })
        );
        assert!(
            doc["content"]
                .as_array()
                .unwrap()
                .iter()
                .all(|node| node["attrs"]["id"].is_string())
        );
    }

    #[test]
    fn fdx_import_rejects_what_is_not_final_draft() {
        assert!(fdx_to_prosemirror("<html><body>hi</body></html>").is_err());
        assert!(fdx_to_prosemirror("<FinalDraft><Content><Paragraph>").is_err());
        assert!(fdx_to_prosemirror("just text").is_err());
        let empty =
            fdx_to_prosemirror("<FinalDraft DocumentType=\"Script\"><Content/></FinalDraft>")
                .unwrap();
        assert_eq!(empty["content"].as_array().unwrap().len(), 1);
    }
}
