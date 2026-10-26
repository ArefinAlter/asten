//! Markdown and plain text into blocks.
//!
//! Markdown: ATX and setext headings, thematic breaks, `# # #` (Asten's scene
//! break), block quotes, bullet and numbered lists, fenced code, paragraphs
//! whose hard-wrapped lines are joined, and the inline forms of
//! [`super::inline`]. Plain text: paragraphs are blocks of lines between blank
//! lines (a hard-wrapped line is joined to the next), a line that is only a row
//! of symbols is a scene break, a line that is only "Chapter IV." or "Part
//! Two" is a heading, and `_italics_` as Project Gutenberg writes them are
//! italics.

use super::blocks::{Block, Inline, marked, tidy};
use super::inline;

const HARD_BREAK: char = '\u{1}';

/// Markdown into blocks.
pub(crate) fn markdown(source: &str) -> Vec<Block> {
    let lines: Vec<String> = source
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(|l| l.replace('\t', "    "))
        .collect();
    blocks(&lines, 0)
}

fn is_rule(trimmed: &str) -> bool {
    let symbols: Vec<char> = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    symbols.len() >= 3
        && matches!(symbols[0], '*' | '-' | '_')
        && symbols.iter().all(|c| *c == symbols[0])
}

fn atx(trimmed: &str) -> Option<(u8, &str)> {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim_end();
    Some((hashes as u8, text))
}

struct Marker {
    ordered: bool,
    indent: usize,
    rest: String,
}

fn marker(line: &str) -> Option<Marker> {
    let indent = line.len() - line.trim_start().len();
    let body = &line[indent..];
    let mut chars = body.chars();
    let first = chars.next()?;
    let (ordered, width) = if matches!(first, '-' | '*' | '+') {
        (false, 1)
    } else if first.is_ascii_digit() {
        let digits = body.chars().take_while(char::is_ascii_digit).count();
        let after = body[digits..].chars().next()?;
        if digits > 9 || !matches!(after, '.' | ')') {
            return None;
        }
        (true, digits + 1)
    } else {
        return None;
    };
    let rest = &body[width..];
    if !rest.starts_with(' ') || rest.trim().is_empty() {
        return None;
    }
    if !ordered && is_rule(body.trim()) {
        return None;
    }
    Some(Marker {
        ordered,
        indent,
        rest: rest.trim_start().to_owned(),
    })
}

fn paragraph(lines: &[String]) -> Block {
    let mut joined = String::new();
    for (index, line) in lines.iter().enumerate() {
        let hard = line.ends_with("  ") || line.ends_with('\\');
        let body = line.trim().trim_end_matches('\\').trim_end();
        joined.push_str(body);
        if index + 1 < lines.len() {
            joined.push(if hard { HARD_BREAK } else { ' ' });
        }
    }
    Block::Paragraph(breaks(inline::parse(&joined)))
}

/// Turn the hard-break sentinel into real breaks.
fn breaks(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::new();
    for item in inlines {
        match item {
            Inline::Text(span) if span.text.contains(HARD_BREAK) => {
                let mut parts = span.text.split(HARD_BREAK).peekable();
                while let Some(part) = parts.next() {
                    out.push(marked(part, &span.marks));
                    if parts.peek().is_some() {
                        out.push(Inline::Break);
                    }
                }
            }
            other => out.push(other),
        }
    }
    tidy(out)
}

fn blocks(lines: &[String], depth: usize) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let flush = |para: &mut Vec<String>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            out.push(paragraph(para));
            para.clear();
        }
    };
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut para, &mut out);
            i += 1;
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            flush(&mut para, &mut out);
            let fence = &trimmed[..3];
            let mut code: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with(fence) {
                code.push(lines[i].clone());
                i += 1;
            }
            i += 1;
            let mut inlines = Vec::new();
            for (n, text) in code.iter().enumerate() {
                if n > 0 {
                    inlines.push(Inline::Break);
                }
                inlines.push(marked(text, &["code"]));
            }
            let inlines = tidy(inlines);
            if !inlines.is_empty() {
                out.push(Block::Paragraph(inlines));
            }
            continue;
        }
        if trimmed == "# # #" {
            flush(&mut para, &mut out);
            out.push(Block::SceneBreak);
            i += 1;
            continue;
        }
        // Setext: text, then a line of = or -.
        if !para.is_empty()
            && (trimmed.chars().all(|c| c == '=')
                || (trimmed.len() >= 2 && trimmed.chars().all(|c| c == '-')))
        {
            let level = if trimmed.starts_with('=') { 1 } else { 2 };
            let heading = paragraph(&para);
            para.clear();
            if let Block::Paragraph(inlines) = heading {
                out.push(Block::Heading(level, inlines));
            }
            i += 1;
            continue;
        }
        if is_rule(trimmed) {
            flush(&mut para, &mut out);
            out.push(Block::Rule);
            i += 1;
            continue;
        }
        if let Some((level, text)) = atx(trimmed) {
            flush(&mut para, &mut out);
            out.push(Block::Heading(level, tidy(inline::parse(text))));
            i += 1;
            continue;
        }
        if trimmed.starts_with('>') && depth < 6 {
            flush(&mut para, &mut out);
            let mut quoted: Vec<String> = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                let body = lines[i].trim_start()[1..].to_owned();
                quoted.push(body.strip_prefix(' ').unwrap_or(&body).to_owned());
                i += 1;
            }
            out.push(Block::Quote(blocks(&quoted, depth + 1)));
            continue;
        }
        if let Some(first) = marker(line) {
            if depth < 6 {
                flush(&mut para, &mut out);
                let ordered = first.ordered;
                let base = first.indent;
                let mut items: Vec<Vec<String>> = Vec::new();
                while i < lines.len() {
                    let current = &lines[i];
                    if let Some(m) = marker(current) {
                        if m.ordered == ordered && m.indent <= base + 1 {
                            items.push(vec![m.rest]);
                            i += 1;
                            continue;
                        }
                        if m.indent > base + 1 && !items.is_empty() {
                            let item = items.last_mut().expect("an item");
                            item.push(current[(base + 2).min(current.len())..].to_owned());
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    if current.trim().is_empty() {
                        // A blank line continues the list only when an indented line follows.
                        let next = lines.get(i + 1);
                        if next.is_some_and(|n| {
                            !n.trim().is_empty()
                                && (n.len() - n.trim_start().len() > base + 1
                                    || marker(n).is_some_and(|m| {
                                        m.ordered == ordered && m.indent <= base + 1
                                    }))
                        }) {
                            if let Some(item) = items.last_mut() {
                                item.push(String::new());
                            }
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    let indent = current.len() - current.trim_start().len();
                    if indent > base && !items.is_empty() {
                        let item = items.last_mut().expect("an item");
                        item.push(current[(base + 2).min(current.len())..].to_owned());
                        i += 1;
                        continue;
                    }
                    // A lazy continuation of the last item's text.
                    if !items.is_empty()
                        && !is_rule(current.trim())
                        && atx(current.trim()).is_none()
                    {
                        let item = items.last_mut().expect("an item");
                        item.push(current.trim().to_owned());
                        i += 1;
                        continue;
                    }
                    break;
                }
                out.push(Block::List {
                    ordered,
                    items: items.iter().map(|item| blocks(item, depth + 1)).collect(),
                });
                continue;
            }
        }
        para.push(line.clone());
        i += 1;
    }
    flush(&mut para, &mut out);
    out
}

// ───────────────────────── plain text ─────────────────────────

const NUMBER_WORDS: [&str; 22] = [
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
];

fn is_numeral(word: &str) -> bool {
    let w = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    if w.is_empty() {
        return false;
    }
    w.chars().all(|c| c.is_ascii_digit())
        || w.chars().all(|c| "ivxlcdm".contains(c))
        || NUMBER_WORDS.contains(&w.as_str())
        || w.starts_with("twenty-")
        || w.starts_with("thirty-")
}

/// A line that is only "Chapter IV.", "PART TWO", "Book 3: The Voyage", or a
/// lone "Prologue".
pub(crate) fn is_chapter_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 90 {
        return false;
    }
    let mut words = trimmed.split_whitespace();
    let first = words
        .next()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .unwrap_or_default();
    match first.as_str() {
        "chapter" | "part" | "book" | "volume" => words.next().is_some_and(is_numeral),
        "prologue" | "epilogue" | "preface" | "foreword" | "afterword" | "introduction" => {
            words.next().is_none()
        }
        _ => false,
    }
}

fn is_symbol_row(line: &str) -> bool {
    let trimmed = line.trim();
    let symbols = trimmed.chars().filter(|c| !c.is_whitespace()).count();
    symbols >= 3
        && trimmed.chars().all(|c| {
            c.is_whitespace() || matches!(c, '*' | '#' | '~' | '=' | '-' | '_' | '•' | '·' | '+')
        })
}

/// Plain text into blocks.
pub(crate) fn plain(source: &str) -> Vec<Block> {
    let text = source.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = text.split('\n').collect();
    let non_empty: Vec<&&str> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return Vec::new();
    }
    let mut lengths: Vec<usize> = non_empty.iter().map(|l| l.trim().chars().count()).collect();
    lengths.sort_unstable();
    let p90 = lengths[(lengths.len() * 9 / 10).min(lengths.len() - 1)];
    let has_blank = lines.iter().any(|l| l.trim().is_empty());
    // Hard-wrapped text keeps its paragraphs between blank lines; text whose
    // lines are long, or that has no blank lines at all, is a paragraph a line.
    let wrapped = has_blank && p90 <= 100;
    let mut out = Vec::new();
    let mut chunk: Vec<&str> = Vec::new();
    let flush = |chunk: &mut Vec<&str>, out: &mut Vec<Block>| {
        if chunk.is_empty() {
            return;
        }
        if chunk.len() == 1 && is_symbol_row(chunk[0]) {
            out.push(Block::SceneBreak);
        } else if chunk.len() == 1 && is_chapter_line(chunk[0]) {
            out.push(Block::Heading(1, tidy(inline::parse(chunk[0].trim()))));
        } else {
            let joined = chunk.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
            out.push(Block::Paragraph(tidy(inline::parse(&joined))));
        }
        chunk.clear();
    };
    for line in lines {
        if line.trim().is_empty() {
            flush(&mut chunk, &mut out);
        } else if wrapped {
            chunk.push(line);
        } else {
            flush(&mut chunk, &mut out);
            chunk.push(line);
            flush(&mut chunk, &mut out);
        }
    }
    flush(&mut chunk, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::blocks::{block_text, text};

    #[test]
    fn markdown_blocks_and_marks() {
        let blocks = markdown(
            "# Title\n\nShe said _never_ twice,\nand left.\n\n> A quoted line\n> and another\n\n- one\n- two *two*\n\n1. first\n2. second\n\n* * *\n\n# # #\n\nEnd",
        );
        assert_eq!(blocks[0], Block::Heading(1, vec![text("Title")]));
        assert_eq!(block_text(&blocks[1]), "She said never twice, and left.");
        assert!(matches!(&blocks[2], Block::Quote(inner) if inner.len() == 1));
        assert!(matches!(&blocks[3], Block::List { ordered: false, items } if items.len() == 2));
        assert!(matches!(&blocks[4], Block::List { ordered: true, items } if items.len() == 2));
        assert_eq!(blocks[5], Block::Rule);
        assert_eq!(blocks[6], Block::SceneBreak);
        assert_eq!(block_text(&blocks[7]), "End");
    }

    #[test]
    fn setext_headings_and_hard_breaks() {
        let blocks = markdown("Chapter One\n===========\n\nline one  \nline two");
        assert_eq!(blocks[0], Block::Heading(1, vec![text("Chapter One")]));
        let Block::Paragraph(inlines) = &blocks[1] else {
            panic!()
        };
        assert!(inlines.contains(&Inline::Break));
    }

    #[test]
    fn asten_markdown_round_trips_its_marks() {
        let blocks = markdown("Plain *slanted* **heavy** ~~struck~~ ++new++");
        let Block::Paragraph(inlines) = &blocks[0] else {
            panic!()
        };
        let marks: Vec<Vec<&str>> = inlines
            .iter()
            .filter_map(|i| match i {
                Inline::Text(s) => Some(s.marks.clone()),
                _ => None,
            })
            .collect();
        assert!(marks.contains(&vec!["italic"]));
        assert!(marks.contains(&vec!["bold"]));
        assert!(marks.contains(&vec!["strike"]));
        assert!(marks.contains(&vec!["insertion"]));
    }

    #[test]
    fn gutenberg_text_is_unwrapped_and_its_chapters_are_headings() {
        let source = "CHAPTER II.\n\nMr. Bennet was among the earliest of those who waited\non Mr. Bingley. He had always intended to visit _him_,\nthough to the last assuring his wife.\n\n*       *       *       *       *\n\nIt was the third.\n";
        let blocks = plain(source);
        assert!(
            matches!(&blocks[0], Block::Heading(1, h) if block_text(&Block::Paragraph(h.clone())) == "CHAPTER II.")
        );
        assert_eq!(
            block_text(&blocks[1]),
            "Mr. Bennet was among the earliest of those who waited on Mr. Bingley. He had always intended to visit him, though to the last assuring his wife."
        );
        let Block::Paragraph(inlines) = &blocks[1] else {
            panic!()
        };
        assert!(
            inlines.iter().any(
                |i| matches!(i, Inline::Text(s) if s.text == "him" && s.marks == vec!["italic"])
            )
        );
        assert_eq!(blocks[2], Block::SceneBreak);
        assert_eq!(block_text(&blocks[3]), "It was the third.");
    }

    #[test]
    fn plain_text_without_blank_lines_is_a_paragraph_a_line() {
        let blocks = plain("First paragraph.\nSecond paragraph.\nThird.");
        assert_eq!(blocks.len(), 3);
    }

    #[test]
    fn chapter_lines_are_recognised_narrowly() {
        for yes in [
            "Chapter 1",
            "CHAPTER XIV.",
            "Chapter I.]",
            "Part Two",
            "Book 3: The Voyage",
            "Prologue",
            "CHAPTER TWENTY-ONE",
        ] {
            assert!(is_chapter_line(yes), "{yes}");
        }
        for no in [
            "Chapter and verse",
            "The chapter ended",
            "Part of the problem",
            "Book club night",
            "Introduction to the sea",
            "",
        ] {
            assert!(!is_chapter_line(no), "{no}");
        }
    }
}
