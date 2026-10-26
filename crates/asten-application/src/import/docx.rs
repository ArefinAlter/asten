//! Word documents into blocks.
//!
//! What is read from the `.docx`: paragraphs, headings (by style name or
//! outline level), block quotes, bullet and numbered lists, italic, bold,
//! underline and strike-through runs, tabs and line breaks, scene-break rows,
//! tracked insertions and deletions (kept as Asten's `insertion` and `deletion`
//! marks, so the author can accept or reject them in the Review tab), and
//! comments (with the range they were attached to). Pictures, footnotes,
//! headers, footers, text boxes and field codes are left out. Nothing is
//! executed and no archive member is written to disk.

use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use quick_xml::{
    Reader, XmlVersion,
    escape::resolve_predefined_entity,
    events::{BytesStart, Event},
};

use super::blocks::{Block, Inline, block_text, inline_text, marked, tidy};
use crate::{ApplicationError, Result};

const MAX_PART: u64 = 96 * 1024 * 1024;

/// A comment found in the document.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ImportedComment {
    /// Index of the top-level block it is anchored to.
    pub block: usize,
    /// Start offset in the block's text (Unicode scalars).
    pub from: u32,
    /// End offset.
    pub to: u32,
    /// Who wrote it.
    pub author: String,
    /// What it says.
    pub body: String,
}

/// The result of reading a `.docx`.
pub(crate) struct DocxImport {
    pub blocks: Vec<Block>,
    pub comments: Vec<ImportedComment>,
    pub insertions: usize,
    pub deletions: usize,
}

fn invalid(message: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::Validation(format!("not a readable Word document: {message}"))
}

fn part(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Result<Option<String>> {
    let Ok(file) = archive.by_name(name) else {
        return Ok(None);
    };
    if file.size() > MAX_PART {
        return Err(invalid(format!("{name} is too large")));
    }
    let mut text = String::new();
    file.take(MAX_PART)
        .read_to_string(&mut text)
        .map_err(invalid)?;
    Ok(Some(text.trim_start_matches('\u{feff}').to_owned()))
}

fn attr(start: &BytesStart, key: &str) -> Option<String> {
    for attribute in start.attributes().flatten() {
        if attribute.key.local_name().as_ref() == key {
            return attribute
                .normalized_value(XmlVersion::Explicit1_0)
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

fn off(start: &BytesStart) -> bool {
    attr(start, "val").is_some_and(|v| matches!(v.as_str(), "0" | "false" | "off" | "none"))
}

#[derive(Default)]
struct Styles {
    /// styleId to (lower-case name, basedOn)
    by_id: HashMap<String, (String, Option<String>)>,
    /// styleId to the numbering the style itself carries (Word's List Bullet).
    numbered: HashMap<String, String>,
}

impl Styles {
    fn read(xml: &str) -> Self {
        let mut styles = Styles::default();
        let mut reader = Reader::from_str(xml);
        let mut current: Option<String> = None;
        let mut name = String::new();
        let mut based: Option<String> = None;
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    let local = e.local_name().as_ref().to_owned();
                    match local.as_str() {
                        "numId" if current.is_some() => {
                            if let (Some(id), Some(num)) = (current.clone(), attr(&e, "val")) {
                                if num != "0" {
                                    styles.numbered.insert(id, num);
                                }
                            }
                        }
                        "style" => {
                            current = attr(&e, "styleId");
                            name.clear();
                            based = None;
                        }
                        "name" if current.is_some() => {
                            name = attr(&e, "val").unwrap_or_default().to_lowercase();
                        }
                        "basedOn" if current.is_some() => based = attr(&e, "val"),
                        _ => {}
                    }
                }
                Ok(Event::End(e)) if e.local_name().as_ref() == "style" => {
                    if let Some(id) = current.take() {
                        styles.by_id.insert(id, (name.clone(), based.take()));
                    }
                }
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        styles
    }

    /// The numbering a style carries, itself or through the styles it is based on.
    fn numbering_of(&self, id: &str) -> Option<String> {
        let mut next = Some(id.to_owned());
        for _ in 0..6 {
            let current = next.take()?;
            if let Some(num) = self.numbered.get(&current) {
                return Some(num.clone());
            }
            next = self
                .by_id
                .get(&current)
                .and_then(|(_, based)| based.clone());
        }
        None
    }

    /// The style's own name and those it is based on, nearest first.
    fn names(&self, id: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut next = Some(id.to_owned());
        while let Some(current) = next.take() {
            if out.len() > 6 {
                break;
            }
            match self.by_id.get(&current) {
                Some((name, based)) => {
                    out.push(name.clone());
                    next = based.clone();
                }
                None => out.push(current.to_lowercase()),
            }
        }
        out
    }
}

/// numId to "is it a numbered (not bulleted) list".
fn numbering_kinds(xml: &str) -> HashMap<String, bool> {
    let mut abstract_ordered: HashMap<String, bool> = HashMap::new();
    let mut nums: HashMap<String, String> = HashMap::new();
    let mut reader = Reader::from_str(xml);
    let mut abs: Option<String> = None;
    let mut seen_level = false;
    let mut num: Option<String> = None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = e.local_name().as_ref().to_owned();
                match local.as_str() {
                    "abstractNum" => {
                        abs = attr(&e, "abstractNumId");
                        seen_level = false;
                    }
                    "numFmt" if abs.is_some() && !seen_level => {
                        let fmt = attr(&e, "val").unwrap_or_default();
                        seen_level = true;
                        if let Some(id) = &abs {
                            abstract_ordered.insert(id.clone(), fmt != "bullet" && fmt != "none");
                        }
                    }
                    "num" => num = attr(&e, "numId"),
                    "abstractNumId" if num.is_some() => {
                        if let (Some(n), Some(a)) = (num.clone(), attr(&e, "val")) {
                            nums.insert(n, a);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                "abstractNum" => abs = None,
                "num" => num = None,
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    nums.into_iter()
        .map(|(n, a)| (n, abstract_ordered.get(&a).copied().unwrap_or(false)))
        .collect()
}

/// comment id to (author, text).
fn read_comments(xml: &str) -> HashMap<String, (String, String)> {
    let mut out = HashMap::new();
    let mut reader = Reader::from_str(xml);
    let mut id: Option<String> = None;
    let mut author = String::new();
    let mut paragraphs: Vec<String> = Vec::new();
    let mut text = String::new();
    let mut in_t = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                "comment" => {
                    id = attr(&e, "id");
                    author = attr(&e, "author").unwrap_or_default();
                    paragraphs.clear();
                    text.clear();
                }
                "t" => in_t = true,
                _ => {}
            },
            Ok(Event::Text(t)) if in_t => text.push_str(&t.xml10_content()),
            Ok(Event::GeneralRef(r)) if in_t => {
                if r.is_char_ref() {
                    if let Ok(Some(c)) = r.resolve_char_ref() {
                        text.push(c);
                    }
                } else if let Some(s) = resolve_predefined_entity(&r.into_inner()) {
                    text.push_str(s);
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                "t" => in_t = false,
                "p" => paragraphs.push(std::mem::take(&mut text)),
                "comment" => {
                    if let Some(id) = id.take() {
                        let body = paragraphs
                            .iter()
                            .map(|p| p.trim())
                            .filter(|p| !p.is_empty())
                            .collect::<Vec<_>>()
                            .join("\n");
                        out.insert(id, (author.clone(), body));
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    out
}

#[derive(Default, Clone)]
struct RunProps {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    hidden: bool,
}

enum Kind {
    Normal,
    Heading(u8),
    Quote,
}

#[derive(Default)]
struct Para {
    inlines: Vec<Inline>,
    style: Vec<String>,
    outline: Option<u8>,
    num: Option<String>,
    /// Numbering that comes from the paragraph's style rather than the paragraph.
    style_num: Option<String>,
    /// Characters so far, for comment offsets.
    chars: u32,
}

struct RawRange {
    id: String,
    start: (usize, u32),
    end: (usize, u32),
}

struct Parser<'a> {
    styles: &'a Styles,
    numbering: &'a HashMap<String, bool>,
    blocks: Vec<Block>,
    pending_list: Option<(bool, Vec<Vec<Block>>)>,
    pending_quote: Vec<Block>,
    /// Per paragraph, in document order: the block it landed in.
    paragraph_blocks: Vec<Option<usize>>,
    /// Per paragraph: characters trimmed from its start.
    paragraph_lead: Vec<u32>,
    insertions: usize,
    deletions: usize,
    para: Option<Para>,
    run: RunProps,
    in_run: bool,
    in_rpr: bool,
    in_ppr: bool,
    change: u32,
    in_t: bool,
    ins: u32,
    del: u32,
    skip: u32,
    starts: HashMap<String, (usize, u32)>,
    ranges: Vec<RawRange>,
}

fn is_skipped(name: &str) -> bool {
    matches!(
        name,
        "drawing" | "pict" | "Fallback" | "object" | "footnotes" | "endnotes"
    )
}

impl Parser<'_> {
    fn here(&self) -> (usize, u32) {
        (
            self.paragraph_lead.len(),
            self.para.as_ref().map_or(0, |p| p.chars),
        )
    }

    fn open(&mut self, e: &BytesStart, start: bool) {
        let name = e.local_name().as_ref().to_owned();
        if self.skip > 0 {
            if start && is_skipped(&name) {
                self.skip += 1;
            }
            return;
        }
        if start && is_skipped(&name) {
            self.skip += 1;
            return;
        }
        let in_run_props = self.in_rpr && self.in_run && self.change == 0;
        match name.as_str() {
            "p" if start => self.para = Some(Para::default()),
            "pPr" if start => self.in_ppr = true,
            "pPrChange" | "rPrChange" | "sectPrChange" | "tblPrChange" if start => {
                self.change += 1;
            }
            "pStyle" if self.in_ppr && self.change == 0 => {
                if let (Some(para), Some(id)) = (self.para.as_mut(), attr(e, "val")) {
                    para.style = self.styles.names(&id);
                    para.style_num = self.styles.numbering_of(&id);
                }
            }
            "outlineLvl" if self.in_ppr && self.change == 0 => {
                if let (Some(para), Some(level)) = (
                    self.para.as_mut(),
                    attr(e, "val").and_then(|v| v.parse::<u8>().ok()),
                ) {
                    para.outline = Some(level);
                }
            }
            "numId" if self.in_ppr && self.change == 0 => {
                if let (Some(para), Some(id)) = (self.para.as_mut(), attr(e, "val")) {
                    if id != "0" {
                        para.num = Some(id);
                    }
                }
            }
            "r" if start => {
                self.in_run = true;
                self.run = RunProps::default();
            }
            "rPr" if start && self.change == 0 => self.in_rpr = true,
            "b" if in_run_props => self.run.bold = !off(e),
            "i" if in_run_props => self.run.italic = !off(e),
            "u" if in_run_props => self.run.underline = !off(e),
            "strike" | "dstrike" if in_run_props => self.run.strike = !off(e),
            "vanish" if in_run_props => self.run.hidden = !off(e),
            "rStyle" if in_run_props => {
                let id = attr(e, "val").unwrap_or_default();
                for name in self.styles.names(&id) {
                    match name.as_str() {
                        "emphasis" | "intense emphasis" | "book title" | "subtle emphasis" => {
                            self.run.italic = true;
                        }
                        "strong" | "intense reference" => self.run.bold = true,
                        _ => {}
                    }
                }
            }
            "ins" | "moveTo" if !self.in_ppr => {
                if start {
                    self.ins += 1;
                }
                self.insertions += 1;
            }
            "del" | "moveFrom" if !self.in_ppr => {
                if start {
                    self.del += 1;
                }
                self.deletions += 1;
            }
            "t" | "delText" if start => {
                self.in_t = self.para.is_some() && self.in_run && !self.run.hidden;
            }
            "tab" if self.in_run => self.push(" "),
            "br" | "cr" if self.in_run => {
                let kind = attr(e, "type").unwrap_or_default();
                if kind != "page" && kind != "column" {
                    self.push_break();
                }
            }
            "noBreakHyphen" if self.in_run => self.push("-"),
            "commentRangeStart" => {
                if let Some(id) = attr(e, "id") {
                    let here = self.here();
                    self.starts.insert(id, here);
                }
            }
            "commentRangeEnd" => {
                if let Some(id) = attr(e, "id") {
                    let end = self.here();
                    if let Some(start) = self.starts.get(&id).copied() {
                        self.ranges.push(RawRange { id, start, end });
                    }
                }
            }
            "commentReference" => {
                if let Some(id) = attr(e, "id") {
                    if !self.ranges.iter().any(|r| r.id == id) {
                        let here = self.here();
                        self.ranges.push(RawRange {
                            id,
                            start: here,
                            end: here,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str) {
        if self.skip > 0 {
            if is_skipped(name) {
                self.skip -= 1;
            }
            return;
        }
        match name {
            "p" => {
                if let Some(para) = self.para.take() {
                    self.finish(para);
                }
            }
            "pPr" => self.in_ppr = false,
            "pPrChange" | "rPrChange" | "sectPrChange" | "tblPrChange" => {
                self.change = self.change.saturating_sub(1);
            }
            "r" => {
                self.in_run = false;
                self.in_t = false;
            }
            "rPr" => self.in_rpr = false,
            "t" | "delText" => self.in_t = false,
            "ins" | "moveTo" if !self.in_ppr => self.ins = self.ins.saturating_sub(1),
            "del" | "moveFrom" if !self.in_ppr => self.del = self.del.saturating_sub(1),
            _ => {}
        }
    }

    fn marks(&self) -> Vec<&'static str> {
        let mut marks = Vec::new();
        if self.run.bold {
            marks.push("bold");
        }
        if self.run.italic {
            marks.push("italic");
        }
        if self.run.underline {
            marks.push("underline");
        }
        if self.run.strike {
            marks.push("strike");
        }
        if self.ins > 0 {
            marks.push("insertion");
        }
        if self.del > 0 {
            marks.push("deletion");
        }
        marks
    }

    fn push(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let marks = self.marks();
        if let Some(para) = self.para.as_mut() {
            para.chars += text.chars().count() as u32;
            para.inlines.push(marked(text, &marks));
        }
    }

    fn push_break(&mut self) {
        if let Some(para) = self.para.as_mut() {
            para.chars += 1;
            para.inlines.push(Inline::Break);
        }
    }

    fn classify(&self, para: &Para) -> Kind {
        for name in &para.style {
            let compact = name.split_whitespace().collect::<Vec<_>>().join(" ");
            if let Some(level) = compact.strip_prefix("heading ") {
                if let Ok(n) = level.trim().parse::<u8>() {
                    return Kind::Heading(n.clamp(1, 6));
                }
            }
            if compact == "title" {
                return Kind::Heading(1);
            }
            if matches!(
                compact.as_str(),
                "quote" | "intense quote" | "block text" | "block quotation"
            ) {
                return Kind::Quote;
            }
        }
        match para.outline {
            Some(level) if level < 6 => Kind::Heading(level + 1),
            _ => Kind::Normal,
        }
    }

    fn close_list(&mut self) {
        if let Some((ordered, items)) = self.pending_list.take() {
            self.blocks.push(Block::List { ordered, items });
        }
    }

    fn close_quote(&mut self) {
        if !self.pending_quote.is_empty() {
            self.blocks
                .push(Block::Quote(std::mem::take(&mut self.pending_quote)));
        }
    }

    fn finish(&mut self, para: Para) {
        let raw = inline_text(&para.inlines);
        let lead = raw.chars().take_while(|c| c.is_whitespace()).count() as u32;
        let inlines = tidy(para.inlines.clone());
        let text = inline_text(&inlines);
        self.paragraph_lead.push(lead);
        if text.trim().is_empty() {
            self.paragraph_blocks.push(None);
            self.close_list();
            self.close_quote();
            return;
        }
        let trimmed = text.trim();
        let symbols = trimmed.chars().filter(|c| !c.is_whitespace()).count();
        let symbol_row = symbols >= 3
            && trimmed.chars().all(|c| {
                c.is_whitespace()
                    || matches!(c, '*' | '#' | '~' | '=' | '\u{2022}' | '\u{b7}' | '+')
            });
        if symbol_row {
            self.close_list();
            self.close_quote();
            self.paragraph_blocks.push(Some(self.blocks.len()));
            self.blocks.push(Block::SceneBreak);
            return;
        }
        // A list item by its own numbering, its style's numbering, or the name
        // of a style Word ships ("List Bullet", "List Number").
        let by_name = para.style.iter().find_map(|name| {
            if name.starts_with("list bullet") {
                Some("style:bullet".to_owned())
            } else if name.starts_with("list number") {
                Some("style:number".to_owned())
            } else {
                None
            }
        });
        let list_num = para.num.clone().or(para.style_num.clone()).or(by_name);
        if let Some(num) = &list_num {
            self.close_quote();
            let ordered = match num.as_str() {
                "style:bullet" => false,
                "style:number" => true,
                other => self.numbering.get(other).copied().unwrap_or(false),
            };
            let item = vec![Block::Paragraph(inlines)];
            match &mut self.pending_list {
                Some((existing, items)) if *existing == ordered => items.push(item),
                _ => {
                    self.close_list();
                    self.pending_list = Some((ordered, vec![item]));
                }
            }
            self.paragraph_blocks.push(Some(self.blocks.len()));
            return;
        }
        self.close_list();
        match self.classify(&para) {
            Kind::Heading(level) => {
                self.close_quote();
                self.paragraph_blocks.push(Some(self.blocks.len()));
                self.blocks.push(Block::Heading(level, inlines));
            }
            Kind::Quote => {
                self.paragraph_blocks.push(Some(self.blocks.len()));
                self.pending_quote.push(Block::Paragraph(inlines));
            }
            Kind::Normal => {
                self.close_quote();
                self.paragraph_blocks.push(Some(self.blocks.len()));
                self.blocks.push(Block::Paragraph(inlines));
            }
        }
    }
}

/// Read a `.docx` from its bytes.
pub(crate) fn read(bytes: Vec<u8>) -> Result<DocxImport> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(invalid)?;
    let document = part(&mut archive, "word/document.xml")?
        .ok_or_else(|| invalid("word/document.xml is missing"))?;
    let styles = part(&mut archive, "word/styles.xml")?
        .map(|xml| Styles::read(&xml))
        .unwrap_or_default();
    let numbering = part(&mut archive, "word/numbering.xml")?
        .map(|xml| numbering_kinds(&xml))
        .unwrap_or_default();
    let comment_text = part(&mut archive, "word/comments.xml")?
        .map(|xml| read_comments(&xml))
        .unwrap_or_default();

    let mut parser = Parser {
        styles: &styles,
        numbering: &numbering,
        blocks: Vec::new(),
        pending_list: None,
        pending_quote: Vec::new(),
        paragraph_blocks: Vec::new(),
        paragraph_lead: Vec::new(),
        insertions: 0,
        deletions: 0,
        para: None,
        run: RunProps::default(),
        in_run: false,
        in_rpr: false,
        in_ppr: false,
        change: 0,
        in_t: false,
        ins: 0,
        del: 0,
        skip: 0,
        starts: HashMap::new(),
        ranges: Vec::new(),
    };

    let mut reader = Reader::from_str(&document);
    let mut saw_document = false;
    loop {
        match reader.read_event().map_err(invalid)? {
            Event::Start(e) => {
                if !saw_document {
                    if e.local_name().as_ref() != "document" {
                        return Err(invalid("the main part is not a Word document"));
                    }
                    saw_document = true;
                }
                parser.open(&e, true);
            }
            Event::Empty(e) => parser.open(&e, false),
            Event::End(e) => {
                let name = e.local_name().as_ref().to_owned();
                parser.close(&name);
            }
            Event::Text(text) => {
                if parser.in_t && parser.skip == 0 {
                    let value = text.xml10_content();
                    parser.push(&value);
                }
            }
            Event::CData(data) => {
                if parser.in_t && parser.skip == 0 {
                    let value = data.into_inner();
                    parser.push(&value);
                }
            }
            Event::GeneralRef(reference) => {
                if parser.in_t && parser.skip == 0 {
                    if reference.is_char_ref() {
                        if let Ok(Some(ch)) = reference.resolve_char_ref() {
                            parser.push(&ch.to_string());
                        }
                    } else if let Some(text) = resolve_predefined_entity(&reference.into_inner()) {
                        parser.push(text);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !saw_document {
        return Err(invalid("the main part is empty"));
    }
    parser.close_list();
    parser.close_quote();

    // Comments: map each range to a block and an offset within it.
    let mut comments = Vec::new();
    for range in &parser.ranges {
        let Some((author, body)) = comment_text.get(&range.id) else {
            continue;
        };
        if body.trim().is_empty() {
            continue;
        }
        let block_of = |paragraph: usize| -> Option<usize> {
            parser
                .paragraph_blocks
                .get(paragraph)
                .copied()
                .flatten()
                .or_else(|| {
                    parser.paragraph_blocks[..paragraph.min(parser.paragraph_blocks.len())]
                        .iter()
                        .rev()
                        .find_map(|b| *b)
                })
        };
        let Some(block) = block_of(range.start.0) else {
            continue;
        };
        let lead = |paragraph: usize| parser.paragraph_lead.get(paragraph).copied().unwrap_or(0);
        let length = parser
            .blocks
            .get(block)
            .map_or(0, |b| block_text(b).chars().count() as u32);
        let from = range
            .start
            .1
            .saturating_sub(lead(range.start.0))
            .min(length);
        let to = if range.end.0 == range.start.0 {
            range.end.1.saturating_sub(lead(range.start.0)).min(length)
        } else {
            length
        };
        comments.push(ImportedComment {
            block,
            from,
            to: to.max(from),
            author: author.clone(),
            body: body.clone(),
        });
    }
    Ok(DocxImport {
        blocks: parser.blocks,
        comments,
        insertions: parser.insertions,
        deletions: parser.deletions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A small .docx with the parts the importer reads.
    fn docx(document: &str, styles: &str, comments: Option<&str>) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        let mut add = |name: &str, body: &str| {
            zip.start_file(name, options).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        };
        add(
            "[Content_Types].xml",
            "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>",
        );
        add("word/document.xml", document);
        add("word/styles.xml", styles);
        if let Some(c) = comments {
            add("word/comments.xml", c);
        }
        zip.finish().unwrap().into_inner()
    }

    const W: &str = "xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"";
    const STYLES: &str = "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:style w:styleId=\"Heading1\"><w:name w:val=\"heading 1\"/></w:style><w:style w:styleId=\"Chapter\"><w:name w:val=\"Chapter Title\"/><w:basedOn w:val=\"Heading1\"/></w:style><w:style w:styleId=\"Quote\"><w:name w:val=\"Quote\"/></w:style></w:styles>";

    #[test]
    fn headings_italics_scene_breaks_and_quotes() {
        let document = format!(
            "<w:document {W}><w:body>\
<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr><w:r><w:t>Chapter One</w:t></w:r></w:p>\
<w:p><w:r><w:t xml:space=\"preserve\">She said </w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>never</w:t></w:r><w:r><w:t>, &amp; meant it.</w:t></w:r></w:p>\
<w:p><w:pPr><w:pStyle w:val=\"Chapter\"/></w:pPr><w:r><w:t>Chapter Two</w:t></w:r></w:p>\
<w:p><w:r><w:t>* * *</w:t></w:r></w:p>\
<w:p><w:pPr><w:pStyle w:val=\"Quote\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Quoted</w:t></w:r></w:p>\
<w:p><w:r><w:t>The end.</w:t></w:r></w:p>\
</w:body></w:document>"
        );
        let imported = read(docx(&document, STYLES, None)).unwrap();
        assert_eq!(imported.blocks.len(), 6);
        assert!(matches!(&imported.blocks[0], Block::Heading(1, _)));
        let Block::Paragraph(runs) = &imported.blocks[1] else {
            panic!()
        };
        assert_eq!(inline_text(runs), "She said never, & meant it.");
        assert!(runs.iter().any(
            |r| matches!(r, Inline::Text(s) if s.text == "never" && s.marks == vec!["italic"])
        ));
        assert!(
            matches!(&imported.blocks[2], Block::Heading(1, _)),
            "a style based on Heading 1 is a heading"
        );
        assert_eq!(imported.blocks[3], Block::SceneBreak);
        assert!(matches!(&imported.blocks[4], Block::Quote(inner) if inner.len() == 1));
    }

    #[test]
    fn tracked_changes_become_marks_and_formatting_changes_are_ignored() {
        let document = format!(
            "<w:document {W}><w:body><w:p>\
<w:r><w:t xml:space=\"preserve\">Kept </w:t></w:r>\
<w:ins w:id=\"1\" w:author=\"A\"><w:r><w:t>added</w:t></w:r></w:ins>\
<w:del w:id=\"2\" w:author=\"A\"><w:r><w:delText xml:space=\"preserve\"> removed</w:delText></w:r></w:del>\
<w:r><w:rPr><w:b/><w:rPrChange w:id=\"3\"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t xml:space=\"preserve\"> heavy</w:t></w:r>\
</w:p></w:body></w:document>"
        );
        let imported = read(docx(&document, STYLES, None)).unwrap();
        assert_eq!((imported.insertions, imported.deletions), (1, 1));
        let Block::Paragraph(runs) = &imported.blocks[0] else {
            panic!()
        };
        let kinds: Vec<(String, Vec<&str>)> = runs
            .iter()
            .filter_map(|r| match r {
                Inline::Text(s) => Some((s.text.clone(), s.marks.clone())),
                _ => None,
            })
            .collect();
        assert!(kinds.contains(&("added".into(), vec!["insertion"])));
        assert!(kinds.contains(&(" removed".into(), vec!["deletion"])));
        assert!(
            kinds.contains(&(" heavy".into(), vec!["bold"])),
            "the old formatting in rPrChange is not applied: {kinds:?}"
        );
    }

    #[test]
    fn comments_keep_their_range_and_their_author() {
        let document = format!(
            "<w:document {W}><w:body>\
<w:p><w:r><w:t>First paragraph.</w:t></w:r></w:p>\
<w:p><w:r><w:t xml:space=\"preserve\">Mira had </w:t></w:r><w:commentRangeStart w:id=\"0\"/><w:r><w:t>green</w:t></w:r><w:commentRangeEnd w:id=\"0\"/><w:r><w:commentReference w:id=\"0\"/></w:r><w:r><w:t xml:space=\"preserve\"> eyes.</w:t></w:r></w:p>\
</w:body></w:document>"
        );
        let comments = format!(
            "<w:comments {W}><w:comment w:id=\"0\" w:author=\"Ida\"><w:p><w:r><w:t>Blue in chapter 2?</w:t></w:r></w:p></w:comment></w:comments>"
        );
        let imported = read(docx(&document, STYLES, Some(&comments))).unwrap();
        assert_eq!(imported.comments.len(), 1);
        let comment = &imported.comments[0];
        assert_eq!(comment.block, 1);
        assert_eq!((comment.from, comment.to), (9, 14));
        assert_eq!(comment.author, "Ida");
        assert_eq!(comment.body, "Blue in chapter 2?");
    }

    #[test]
    fn lists_and_hidden_text_and_drawings() {
        let document = format!(
            "<w:document {W}><w:body>\
<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p>\
<w:p><w:pPr><w:numPr><w:numId w:val=\"1\"/></w:numPr></w:pPr><w:r><w:t>two</w:t></w:r></w:p>\
<w:p><w:r><w:rPr><w:vanish/></w:rPr><w:t>secret</w:t></w:r><w:r><w:drawing><w:t>caption text</w:t></w:drawing></w:r><w:r><w:t>shown</w:t></w:r></w:p>\
</w:body></w:document>"
        );
        let imported = read(docx(&document, STYLES, None)).unwrap();
        assert!(matches!(&imported.blocks[0], Block::List { items, .. } if items.len() == 2));
        assert_eq!(block_text(&imported.blocks[1]), "shown");
    }

    #[test]
    fn a_list_whose_numbering_lives_in_its_style_is_a_list() {
        let styles = "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:style w:styleId=\"ListBullet\"><w:name w:val=\"List Bullet\"/><w:pPr><w:numPr><w:numId w:val=\"7\"/></w:numPr></w:pPr></w:style><w:style w:styleId=\"ListNumber\"><w:name w:val=\"List Number\"/></w:style></w:styles>";
        let document = format!(
            "<w:document {W}><w:body>\
<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/></w:pPr><w:r><w:t>Rope</w:t></w:r></w:p>\
<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/></w:pPr><w:r><w:t>Lantern</w:t></w:r></w:p>\
<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/></w:pPr><w:r><w:t>First</w:t></w:r></w:p>\
</w:body></w:document>"
        );
        let imported = read(docx(&document, styles, None)).unwrap();
        assert!(
            matches!(&imported.blocks[0], Block::List { ordered: false, items } if items.len() == 2)
        );
        assert!(
            matches!(&imported.blocks[1], Block::List { ordered: true, items } if items.len() == 1)
        );
    }

    #[test]
    fn something_that_is_not_a_document_is_refused() {
        assert!(read(b"not a zip".to_vec()).is_err());
        let wrong = docx("<x/>", STYLES, None);
        assert!(read(wrong).is_err());
    }
}
