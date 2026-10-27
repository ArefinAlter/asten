//! HTML into blocks, without a browser: a tolerant tag scanner that keeps
//! paragraphs, headings, block quotes, lists, rules and the emphasis a writer
//! made (`<em>`, `<strong>`, `<u>`, `<s>`, `<ins>`, `<del>`), reads the common
//! entities, and drops scripts, styles and everything in `<head>`.

use super::blocks::{Block, Inline, marked, tidy};

enum Frame {
    Root(Vec<Block>),
    Quote(Vec<Block>),
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
        current: Option<Vec<Block>>,
    },
}

struct Builder {
    frames: Vec<Frame>,
    current: Vec<Inline>,
    marks: Vec<&'static str>,
    heading: Option<u8>,
    pre: u32,
}

impl Builder {
    fn new() -> Self {
        Self {
            frames: vec![Frame::Root(Vec::new())],
            current: Vec::new(),
            marks: Vec::new(),
            heading: None,
            pre: 0,
        }
    }

    fn emit(&mut self, block: Block) {
        match self.frames.last_mut().expect("a root frame") {
            Frame::Root(blocks) | Frame::Quote(blocks) => blocks.push(block),
            Frame::List { current, .. } => current.get_or_insert_with(Vec::new).push(block),
        }
    }

    fn flush(&mut self) {
        let inlines = tidy(std::mem::take(&mut self.current));
        let heading = self.heading.take();
        if inlines.is_empty() {
            return;
        }
        match heading {
            Some(level) => self.emit(Block::Heading(level, inlines)),
            None => self.emit(Block::Paragraph(inlines)),
        }
    }

    fn text(&mut self, raw: &str) {
        let decoded = decode(raw);
        let text = if self.pre > 0 {
            decoded
        } else {
            let mut collapsed = String::with_capacity(decoded.len());
            let mut space = false;
            for c in decoded.chars() {
                if c.is_whitespace() && c != '\u{a0}' {
                    if !space {
                        collapsed.push(' ');
                    }
                    space = true;
                } else {
                    collapsed.push(if c == '\u{a0}' { ' ' } else { c });
                    space = false;
                }
            }
            collapsed
        };
        if text.trim().is_empty() && self.current.is_empty() {
            return;
        }
        self.current.push(marked(&text, &self.marks));
    }

    fn open_item(&mut self) {
        self.flush();
        if let Some(Frame::List { items, current, .. }) = self.frames.last_mut() {
            if let Some(done) = current.take() {
                items.push(done);
            }
            *current = Some(Vec::new());
        }
    }

    fn close_item(&mut self) {
        self.flush();
        if let Some(Frame::List { items, current, .. }) = self.frames.last_mut() {
            if let Some(done) = current.take() {
                items.push(done);
            }
        }
    }

    fn close_frame(&mut self) {
        self.flush();
        if self.frames.len() < 2 {
            return;
        }
        match self.frames.pop().expect("a frame") {
            Frame::Quote(blocks) => {
                if !blocks.is_empty() {
                    self.emit(Block::Quote(blocks));
                }
            }
            Frame::List {
                ordered,
                mut items,
                current,
            } => {
                if let Some(done) = current {
                    items.push(done);
                }
                items.retain(|item| !item.is_empty());
                if !items.is_empty() {
                    self.emit(Block::List { ordered, items });
                }
            }
            Frame::Root(_) => {}
        }
    }

    fn mark(&mut self, name: &'static str) {
        self.marks.push(name);
    }

    fn unmark(&mut self, name: &'static str) {
        if let Some(at) = self.marks.iter().rposition(|m| *m == name) {
            self.marks.remove(at);
        }
    }
}

fn inline_mark(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "em" | "i" | "cite" | "dfn" => "italic",
        "strong" | "b" => "bold",
        "u" | "ins_u" => "underline",
        "s" | "strike" => "strike",
        "del" => "deletion",
        "ins" => "insertion",
        "code" | "tt" | "kbd" | "samp" => "code",
        _ => return None,
    })
}

const SKIPPED: [&str; 7] = [
    "script", "style", "head", "title", "svg", "noscript", "template",
];
const BLOCKS: [&str; 22] = [
    "p",
    "div",
    "section",
    "article",
    "header",
    "footer",
    "main",
    "nav",
    "aside",
    "figure",
    "figcaption",
    "table",
    "tr",
    "td",
    "th",
    "tbody",
    "thead",
    "address",
    "center",
    "body",
    "html",
    "dl",
];

/// HTML into blocks.
pub(crate) fn html(source: &str) -> Vec<Block> {
    let mut builder = Builder::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut skip: Option<String> = None;
    let mut span_marks: Vec<Vec<&'static str>> = Vec::new();
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let end = source[i..].find('<').map_or(source.len(), |n| i + n);
            if skip.is_none() {
                builder.text(&source[i..end]);
            }
            i = end;
            continue;
        }
        let rest = &source[i..];
        if rest.starts_with("<!--") {
            i += rest.find("-->").map_or(rest.len(), |n| n + 3);
            continue;
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            i += rest.find('>').map_or(rest.len(), |n| n + 1);
            continue;
        }
        let starts_tag = rest
            .as_bytes()
            .get(1)
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'/');
        let Some(close) = tag_end(rest).filter(|_| starts_tag) else {
            // A stray "<": treat it as text.
            if skip.is_none() {
                builder.text("<");
            }
            i += 1;
            continue;
        };
        let inside = &rest[1..close];
        i += close + 1;
        let closing = inside.starts_with('/');
        let body = inside.trim_start_matches('/').trim_end_matches('/');
        let name_end = body.find(|c: char| c.is_whitespace()).unwrap_or(body.len());
        let name = body[..name_end].to_ascii_lowercase();
        let attrs = &body[name_end..];
        if let Some(skipping) = &skip {
            if closing && &name == skipping {
                skip = None;
            }
            continue;
        }
        if !closing && SKIPPED.contains(&name.as_str()) && !inside.ends_with('/') {
            skip = Some(name);
            continue;
        }
        if closing {
            end_tag(&mut builder, &name, &mut span_marks);
        } else {
            start_tag(&mut builder, &name, attrs, &mut span_marks);
        }
    }
    while builder.frames.len() > 1 {
        builder.close_frame();
    }
    builder.flush();
    match builder.frames.pop() {
        Some(Frame::Root(blocks)) => blocks,
        _ => Vec::new(),
    }
}

/// The index of the `>` that ends the tag at the start of `rest`, honouring quotes.
fn tag_end(rest: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (index, c) in rest.char_indices().skip(1) {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(index),
            (None, '<') => return None,
            _ => {}
        }
    }
    None
}

fn style_marks(attrs: &str) -> Vec<&'static str> {
    let lower = attrs.to_ascii_lowercase();
    let mut marks = Vec::new();
    if lower.contains("font-style:italic") || lower.contains("font-style: italic") {
        marks.push("italic");
    }
    if lower.contains("font-weight:bold")
        || lower.contains("font-weight: bold")
        || lower.contains("font-weight:700")
        || lower.contains("font-weight: 700")
    {
        marks.push("bold");
    }
    if lower.contains("text-decoration:underline") || lower.contains("text-decoration: underline") {
        marks.push("underline");
    }
    marks
}

fn start_tag(b: &mut Builder, name: &str, attrs: &str, span_marks: &mut Vec<Vec<&'static str>>) {
    if let Some(mark) = inline_mark(name) {
        b.mark(mark);
        return;
    }
    match name {
        "span" | "font" | "a" => {
            let marks = style_marks(attrs);
            for m in &marks {
                b.mark(m);
            }
            span_marks.push(marks);
        }
        "br" => b.current.push(Inline::Break),
        "hr" => {
            b.flush();
            let scene = attrs.contains("data-scene");
            b.emit(if scene {
                Block::SceneBreak
            } else {
                Block::Rule
            });
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            b.flush();
            b.heading = Some(name.as_bytes()[1] - b'0');
        }
        "blockquote" => {
            b.flush();
            b.frames.push(Frame::Quote(Vec::new()));
        }
        "ul" | "ol" => {
            b.flush();
            b.frames.push(Frame::List {
                ordered: name == "ol",
                items: Vec::new(),
                current: None,
            });
        }
        "li" => b.open_item(),
        "pre" => {
            b.flush();
            b.pre += 1;
        }
        n if BLOCKS.contains(&n) => b.flush(),
        _ => {}
    }
}

fn end_tag(b: &mut Builder, name: &str, span_marks: &mut Vec<Vec<&'static str>>) {
    if let Some(mark) = inline_mark(name) {
        b.unmark(mark);
        return;
    }
    match name {
        "span" | "font" | "a" => {
            if let Some(marks) = span_marks.pop() {
                for m in marks {
                    b.unmark(m);
                }
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" => b.flush(),
        "blockquote" => b.close_frame(),
        "ul" | "ol" => b.close_frame(),
        "li" => b.close_item(),
        "pre" => {
            b.flush();
            b.pre = b.pre.saturating_sub(1);
        }
        n if BLOCKS.contains(&n) => b.flush(),
        _ => {}
    }
}

/// Decode the entities that turn up in manuscripts and exports.
pub(crate) fn decode(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let Some(end) = rest.find(';').filter(|&e| e <= 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            "mdash" => Some('\u{2014}'),
            "ndash" => Some('\u{2013}'),
            "hellip" => Some('\u{2026}'),
            "lsquo" => Some('\u{2018}'),
            "rsquo" => Some('\u{2019}'),
            "ldquo" => Some('\u{201c}'),
            "rdquo" => Some('\u{201d}'),
            _ => entity
                .strip_prefix('#')
                .and_then(|n| {
                    n.strip_prefix(['x', 'X'])
                        .map(|h| u32::from_str_radix(h, 16).ok())
                        .unwrap_or_else(|| n.parse::<u32>().ok())
                })
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::blocks::block_text;

    #[test]
    fn paragraphs_headings_and_emphasis() {
        let blocks = html(
            "<!doctype html><html><head><title>x</title><style>p{}</style></head><body><h1>Title</h1><p>She said <em>never</em> &amp; <b>twice</b>.</p><p>Second<br>line</p></body></html>",
        );
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], Block::Heading(1, _)));
        assert_eq!(block_text(&blocks[1]), "She said never & twice.");
        let Block::Paragraph(inlines) = &blocks[1] else {
            panic!()
        };
        assert!(inlines.iter().any(
            |i| matches!(i, Inline::Text(s) if s.text == "never" && s.marks == vec!["italic"])
        ));
        assert!(matches!(&blocks[2], Block::Paragraph(i) if i.contains(&Inline::Break)));
    }

    #[test]
    fn paragraph_boundaries_survive_without_blank_lines() {
        let blocks = html("<p>one</p><p>two</p><p>three</p>");
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn lists_quotes_rules_and_tracked_marks() {
        let blocks = html(
            "<blockquote><p>quoted</p></blockquote><ul><li>a</li><li>b <ins>new</ins> <del>old</del></li></ul><hr><hr data-scene=\"true\"><ol><li>x</li></ol>",
        );
        assert!(matches!(&blocks[0], Block::Quote(_)));
        assert!(matches!(&blocks[1], Block::List { ordered: false, items } if items.len() == 2));
        assert_eq!(blocks[2], Block::Rule);
        assert_eq!(blocks[3], Block::SceneBreak);
        assert!(matches!(&blocks[4], Block::List { ordered: true, .. }));
        let Block::List { items, .. } = &blocks[1] else {
            panic!()
        };
        let Block::Paragraph(second) = &items[1][0] else {
            panic!()
        };
        assert!(
            second
                .iter()
                .any(|i| matches!(i, Inline::Text(s) if s.marks == vec!["insertion"]))
        );
        assert!(
            second
                .iter()
                .any(|i| matches!(i, Inline::Text(s) if s.marks == vec!["deletion"]))
        );
    }

    #[test]
    fn entities_and_stray_angle_brackets() {
        assert_eq!(
            decode("a &lt; b &#8212; c &#x2019; &unknown; &"),
            "a < b \u{2014} c \u{2019} &unknown; &"
        );
        let blocks = html("<p>1 < 2 and 3 > 2</p>");
        assert_eq!(block_text(&blocks[0]), "1 < 2 and 3 > 2");
    }

    #[test]
    fn scripts_are_dropped() {
        let blocks = html("<p>keep</p><script>alert('<p>no</p>')</script><p>this</p>");
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn inline_styles_count() {
        let blocks = html("<p>a <span style=\"font-style: italic\">slanted</span> word</p>");
        let Block::Paragraph(inlines) = &blocks[0] else {
            panic!()
        };
        assert!(inlines.iter().any(
            |i| matches!(i, Inline::Text(s) if s.text == "slanted" && s.marks == vec!["italic"])
        ));
    }
}
