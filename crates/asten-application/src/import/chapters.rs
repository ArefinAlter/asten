//! Splitting one long import into chapters.
//!
//! A novel pasted or imported as a single file belongs in the binder as one
//! manuscript with a document per chapter: a page that is a whole book is slow
//! to edit and useless to navigate. The split happens at the shallowest heading
//! level that occurs often enough to be chapters, so a title at level 1 above
//! chapters at level 2 does not split the book into one-block pieces.

use super::{
    blocks::{Block, block_text},
    docx::ImportedComment,
};

/// How to split.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Mode {
    /// Do not split.
    #[default]
    Off,
    /// Split when there are at least three chapter headings.
    Auto,
    /// Split when there are at least two.
    Chapters,
}

/// One document of an import.
pub(crate) struct Part {
    pub title: String,
    pub blocks: Vec<Block>,
    pub comments: Vec<ImportedComment>,
}

fn heading_title(block: &Block) -> String {
    let raw = block_text(block).replace('\n', " ");
    let mut title = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    // A stray closing bracket from a scanned illustration caption.
    while title.ends_with(']') && !title.contains('[') {
        title.pop();
    }
    while title.ends_with(')') && !title.contains('(') {
        title.pop();
    }
    let title = title.trim().to_owned();
    if title.is_empty() {
        return "Untitled".into();
    }
    if title.chars().count() > 120 {
        let cut: String = title.chars().take(117).collect();
        return format!("{cut}…");
    }
    title
}

/// The result of a split.
pub(crate) struct Split {
    /// A title found at the very top (a heading with nothing under it, before
    /// the chapters): the manuscript's name.
    pub title: Option<String>,
    pub parts: Vec<Part>,
}

/// Split into chapter parts, or `None` when the text does not look like
/// chapters (then the caller keeps one document).
pub(crate) fn split(blocks: &[Block], comments: &[ImportedComment], mode: Mode) -> Option<Split> {
    let needed = match mode {
        Mode::Off => return None,
        Mode::Auto => 3,
        Mode::Chapters => 2,
    };
    let level = (1..=3u8).find(|level| {
        blocks
            .iter()
            .filter(|b| matches!(b, Block::Heading(l, _) if l == level))
            .count()
            >= needed
    })?;
    let starts: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| matches!(b, Block::Heading(l, _) if *l == level))
        .map(|(i, _)| i)
        .collect();
    let mut parts = Vec::new();
    let first = starts[0];
    let front: Vec<Block> = blocks[..first].to_vec();
    // Only headings before the chapters (a title, the manuscript's name) are a
    // title, not a front-matter document.
    let mut title_from_front = None;
    if !front.is_empty() && front.iter().all(|b| matches!(b, Block::Heading(..))) {
        title_from_front = Some(heading_title(&front[0]));
    } else if front.iter().any(|b| !block_text(b).trim().is_empty()) {
        parts.push(Part {
            title: "Front matter".into(),
            blocks: front,
            comments: comments
                .iter()
                .filter(|c| c.block < first)
                .cloned()
                .collect(),
        });
    }
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(blocks.len());
        let body: Vec<Block> = blocks[start + 1..end].to_vec();
        let mut mine = Vec::new();
        for comment in comments
            .iter()
            .filter(|c| c.block >= start && c.block < end)
        {
            let mut c = comment.clone();
            if c.block == start {
                // On the heading itself: keep it on the first block.
                c.block = 0;
                c.from = 0;
                c.to = 0;
            } else {
                c.block -= start + 1;
            }
            if !body.is_empty() {
                mine.push(c);
            }
        }
        parts.push(Part {
            title: heading_title(&blocks[start]),
            blocks: body,
            comments: mine,
        });
    }
    // A heading at the very top with nothing under it is the book's title,
    // not a chapter.
    let mut title = title_from_front;
    if starts[0] == 0 && parts.first().is_some_and(|p| p.blocks.is_empty()) && parts.len() > needed
    {
        let first = parts.remove(0);
        title = Some(first.title);
    }
    Some(Split { title, parts })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::blocks::text;

    fn heading(level: u8, title: &str) -> Block {
        Block::Heading(level, vec![text(title)])
    }
    fn para(text_: &str) -> Block {
        Block::Paragraph(vec![text(text_)])
    }

    #[test]
    fn a_book_title_above_chapters_does_not_split_at_the_title() {
        let blocks = vec![
            heading(1, "The Book"),
            para("An epigraph."),
            heading(2, "One"),
            para("a"),
            heading(2, "Two"),
            para("b"),
            heading(2, "Three"),
            para("c"),
        ];
        let parts = split(&blocks, &[], Mode::Auto).unwrap().parts;
        let titles: Vec<&str> = parts.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, vec!["Front matter", "One", "Two", "Three"]);
        assert_eq!(parts[1].blocks.len(), 1);
    }

    #[test]
    fn auto_wants_three_chapters_and_explicit_wants_two() {
        let blocks = vec![heading(1, "A"), para("x"), heading(1, "B"), para("y")];
        assert!(split(&blocks, &[], Mode::Auto).is_none());
        assert_eq!(split(&blocks, &[], Mode::Chapters).unwrap().parts.len(), 2);
        assert!(split(&blocks, &[], Mode::Off).is_none());
        assert!(split(&[para("no headings")], &[], Mode::Chapters).is_none());
    }

    #[test]
    fn a_lone_title_at_the_top_names_the_manuscript() {
        let blocks = vec![
            heading(1, "The Salt Road"),
            heading(1, "One"),
            para("a"),
            heading(1, "Two"),
            para("b"),
            heading(1, "Three"),
            para("c"),
        ];
        let split = split(&blocks, &[], Mode::Auto).unwrap();
        assert_eq!(split.title.as_deref(), Some("The Salt Road"));
        assert_eq!(split.parts.len(), 3);
    }

    #[test]
    fn headings_alone_before_the_chapters_are_a_title_not_a_document() {
        let blocks = vec![
            heading(1, "The Salt Road"),
            heading(2, "One"),
            para("a"),
            heading(2, "Two"),
            para("b"),
            heading(2, "Three"),
            para("c"),
        ];
        let split = split(&blocks, &[], Mode::Auto).unwrap();
        assert_eq!(split.title.as_deref(), Some("The Salt Road"));
        assert_eq!(split.parts.len(), 3);
        assert_eq!(split.parts[0].title, "One");
    }

    #[test]
    fn titles_are_cleaned_and_comments_follow_their_blocks() {
        let blocks = vec![
            heading(1, "Chapter I.]"),
            para("one"),
            para("two"),
            heading(1, "Chapter II."),
            para("three"),
        ];
        let comments = vec![ImportedComment {
            block: 4,
            from: 1,
            to: 3,
            author: "A".into(),
            body: "b".into(),
        }];
        let parts = split(&blocks, &comments, Mode::Chapters).unwrap().parts;
        assert_eq!(parts[0].title, "Chapter I.");
        assert_eq!(parts[1].comments.len(), 1);
        assert_eq!(parts[1].comments[0].block, 0);
        assert!(parts[0].comments.is_empty());
    }
}
