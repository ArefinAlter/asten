//! Screenplay elements, read the way a Fountain reader reads them.
//!
//! The desk has no blank lines: every paragraph is a block. So the classifier
//! takes a run of paragraphs and reads Fountain's conventions off the text —
//! an uppercase cue followed by a line is a character, a parenthesised line
//! after a cue is a parenthetical, `INT.` opens a scene heading, uppercase
//! ending in `TO:` is a transition — and honours Fountain's forcing marks
//! (`!` action, `.` heading, `@` character, `>` transition) where the bare
//! text would mislead. The Fountain compile, the Fountain importer and the
//! Final Draft (FDX) paths all share it, so a script keeps its shape through
//! every export and import.

use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenplayElement {
    SceneHeading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
    General,
}

impl ScreenplayElement {
    /// The `Type` attribute Final Draft gives a paragraph of this element.
    #[must_use]
    pub fn fdx_type(self) -> &'static str {
        match self {
            ScreenplayElement::SceneHeading => "Scene Heading",
            ScreenplayElement::Action => "Action",
            ScreenplayElement::Character => "Character",
            ScreenplayElement::Parenthetical => "Parenthetical",
            ScreenplayElement::Dialogue => "Dialogue",
            ScreenplayElement::Transition => "Transition",
            ScreenplayElement::General => "General",
        }
    }

    /// The element a Final Draft `Type` attribute names. Types Fountain has no
    /// word for (Shot, Cast List, New Act…) read as `General`.
    #[must_use]
    pub fn from_fdx_type(value: &str) -> Self {
        match value.trim() {
            "Scene Heading" => ScreenplayElement::SceneHeading,
            "Action" => ScreenplayElement::Action,
            "Character" => ScreenplayElement::Character,
            "Parenthetical" => ScreenplayElement::Parenthetical,
            "Dialogue" => ScreenplayElement::Dialogue,
            "Transition" => ScreenplayElement::Transition,
            _ => ScreenplayElement::General,
        }
    }

    /// The Fountain mark that forces this element, when one exists.
    #[must_use]
    pub fn forcing_mark(self) -> Option<char> {
        match self {
            ScreenplayElement::SceneHeading => Some('.'),
            ScreenplayElement::Character => Some('@'),
            ScreenplayElement::Transition => Some('>'),
            ScreenplayElement::Action => Some('!'),
            _ => None,
        }
    }
}

/// `INT.`, `EXT.`, `EST.`, `INT./EXT.`, `INT/EXT.` or `I/E.` (any case), each
/// followed by a period or a space: the shape of a scene heading.
#[must_use]
pub fn is_slugline(text: &str) -> bool {
    let upper = text.trim().to_uppercase();
    ["INT./EXT", "INT/EXT", "INT", "EXT", "EST", "I/E"]
        .iter()
        .any(|prefix| {
            upper
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.') || rest.starts_with(' '))
        })
}

/// Uppercase text ending in `TO:`, or a line forced with a leading `>` that
/// is not centred text (`> like this <`).
#[must_use]
pub fn is_transition(text: &str) -> bool {
    let text = text.trim();
    if let Some(forced) = text.strip_prefix('>') {
        let forced = forced.trim();
        return !forced.is_empty() && !forced.ends_with('<');
    }
    is_uppercase(text) && text.ends_with("TO:")
}

/// An uppercase name, optionally with an extension such as `(V.O.)` and a
/// dual-dialogue caret, or a name forced with `@`. Sentence punctuation at the
/// end (`BANG.`, `FADE IN:`) marks an action line, not a cue.
#[must_use]
pub fn is_character_cue(text: &str) -> bool {
    let text = text.trim().trim_end_matches('^').trim_end();
    if let Some(forced) = text.strip_prefix('@') {
        return !forced.trim().is_empty();
    }
    let name = text.split('(').next().unwrap_or_default().trim();
    if name.is_empty() || !is_uppercase(name) {
        return false;
    }
    !matches!(text.chars().last(), Some('.' | '!' | '?' | ':' | ',' | ';'))
}

#[must_use]
pub fn is_parenthetical(text: &str) -> bool {
    let text = text.trim();
    text.len() > 1 && text.starts_with('(') && text.ends_with(')')
}

/// Has a cased capital and no lowercase letter. Caseless scripts never pass,
/// so a Japanese or Thai line is never mistaken for a cue; `@` forces one.
fn is_uppercase(text: &str) -> bool {
    text.chars().any(char::is_uppercase) && !text.chars().any(char::is_lowercase)
}

/// One element per paragraph, read in order so that context carries: the
/// paragraph after a cue is dialogue, a parenthesised one after dialogue is a
/// parenthetical, and a cue only counts when something follows it.
#[must_use]
pub fn classify_screenplay(paragraphs: &[&str]) -> Vec<ScreenplayElement> {
    let mut kinds: Vec<ScreenplayElement> = Vec::with_capacity(paragraphs.len());
    for (index, raw) in paragraphs.iter().enumerate() {
        let previous = index.checked_sub(1).map(|earlier| kinds[earlier]);
        let has_next = paragraphs
            .get(index + 1)
            .is_some_and(|next| !next.trim().is_empty());
        kinds.push(classify_paragraph(raw, previous, has_next));
    }
    kinds
}

fn classify_paragraph(
    raw: &str,
    previous: Option<ScreenplayElement>,
    has_next: bool,
) -> ScreenplayElement {
    use ScreenplayElement::{Action, Character, Dialogue, Parenthetical, SceneHeading, Transition};
    let text = raw.trim();
    if text.is_empty() {
        return Action;
    }
    if text.starts_with('!') {
        return Action;
    }
    if text.starts_with('.') && !text.starts_with("..") && text.len() > 1 {
        return SceneHeading;
    }
    if text.starts_with('@') && text.len() > 1 {
        return Character;
    }
    if text.starts_with('>') && is_transition(text) {
        return Transition;
    }
    if matches!(previous, Some(Character | Parenthetical)) {
        return if is_parenthetical(text) {
            Parenthetical
        } else {
            Dialogue
        };
    }
    if previous == Some(Dialogue) && is_parenthetical(text) {
        return Parenthetical;
    }
    if is_transition(text) {
        return Transition;
    }
    if is_slugline(text) {
        return SceneHeading;
    }
    if has_next && is_character_cue(text) {
        return Character;
    }
    Action
}

/// The text of an element without its Fountain forcing mark or dual-dialogue
/// caret, for formats that carry the element type explicitly.
#[must_use]
pub fn screenplay_text(text: &str, element: ScreenplayElement) -> &str {
    let text = text.trim();
    let stripped = match element {
        ScreenplayElement::SceneHeading if !text.starts_with("..") => text.strip_prefix('.'),
        ScreenplayElement::Character => text.strip_prefix('@'),
        ScreenplayElement::Action => text.strip_prefix('!'),
        ScreenplayElement::Transition => text.strip_prefix('>'),
        _ => None,
    };
    let text = stripped.unwrap_or(text);
    let text = if element == ScreenplayElement::Character {
        text.trim_end_matches('^')
    } else {
        text
    };
    text.trim()
}

/// Paragraph texts whose elements are already known, written so that
/// [`classify_screenplay`] reads them back the same way: a forcing mark is
/// added only where the bare text would be misread, which is how a Fountain
/// writer would disambiguate by hand.
#[must_use]
pub fn force_screenplay_marks(elements: &[(ScreenplayElement, String)]) -> Vec<String> {
    let mut texts: Vec<String> = elements.iter().map(|(_, text)| text.clone()).collect();
    // One mark per pass: forcing an earlier paragraph changes how the ones
    // after it read, so the run is classified again before the next decision.
    for _ in 0..=elements.len() {
        let kinds = classify_screenplay(&texts.iter().map(String::as_str).collect::<Vec<_>>());
        let misread = elements
            .iter()
            .enumerate()
            .find_map(|(index, (intended, _))| {
                let mark = intended.forcing_mark()?;
                (kinds[index] != *intended && !texts[index].trim_start().starts_with(mark))
                    .then_some((index, mark))
            });
        let Some((index, mark)) = misread else {
            break;
        };
        let gap = if mark == '>' { " " } else { "" };
        texts[index] = format!("{mark}{gap}{}", texts[index].trim_start());
    }
    texts
}

enum FountainBlock {
    Element(ScreenplayElement, String),
    Heading(u32, String),
    Rule,
}

/// Fountain text as a ProseMirror document: one paragraph per element, with
/// a speech kept together as one paragraph, sections as headings and page
/// breaks as rules. The title page is dropped, notes and the boneyard are
/// removed, and forcing marks are added wherever the paragraph form would
/// otherwise lose an element's meaning.
#[must_use]
pub fn fountain_to_prosemirror(fountain: &str) -> Value {
    let text = fountain.replace("\r\n", "\n").replace('\r', "\n");
    let text = remove_between(&text, "/*", "*/");
    let text = remove_between(&text, "[[", "]]");
    let body = skip_title_page(&text);
    let mut blocks: Vec<FountainBlock> = Vec::new();
    for block in body.split("\n\n") {
        let lines: Vec<&str> = block
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        let Some(first) = lines.first().copied() else {
            continue;
        };
        if lines.len() == 1 && first.len() >= 3 && first.chars().all(|ch| ch == '=')
            || lines.len() == 1 && (first == "* * *" || first == "---")
        {
            blocks.push(FountainBlock::Rule);
            continue;
        }
        if first.starts_with('#') {
            for line in lines {
                match parse_section(line) {
                    Some((level, title)) => blocks.push(FountainBlock::Heading(level, title)),
                    None => blocks.push(FountainBlock::Element(
                        ScreenplayElement::Action,
                        line.to_owned(),
                    )),
                }
            }
            continue;
        }
        if lines.len() > 1
            && is_character_cue(first)
            && !is_slugline(first)
            && !is_transition(first)
        {
            blocks.push(FountainBlock::Element(
                ScreenplayElement::Character,
                first.to_owned(),
            ));
            let mut speech: Vec<&str> = Vec::new();
            for line in &lines[1..] {
                if is_parenthetical(line) {
                    flush_speech(&mut speech, &mut blocks);
                    blocks.push(FountainBlock::Element(
                        ScreenplayElement::Parenthetical,
                        (*line).to_owned(),
                    ));
                } else {
                    speech.push(line);
                }
            }
            flush_speech(&mut speech, &mut blocks);
            continue;
        }
        let element = if lines.len() > 1 || first.starts_with('!') {
            ScreenplayElement::Action
        } else if first.starts_with('.') && !first.starts_with("..") && first.len() > 1 {
            ScreenplayElement::SceneHeading
        } else if first.starts_with('@') {
            ScreenplayElement::Character
        } else if is_transition(first) {
            ScreenplayElement::Transition
        } else if is_slugline(first) {
            ScreenplayElement::SceneHeading
        } else {
            ScreenplayElement::Action
        };
        blocks.push(FountainBlock::Element(element, lines.join("\n")));
    }
    screenplay_blocks_to_prosemirror(blocks)
}

fn flush_speech(speech: &mut Vec<&str>, blocks: &mut Vec<FountainBlock>) {
    if speech.is_empty() {
        return;
    }
    blocks.push(FountainBlock::Element(
        ScreenplayElement::Dialogue,
        speech.join("\n"),
    ));
    speech.clear();
}

fn parse_section(line: &str) -> Option<(u32, String)> {
    let mut level = 0u32;
    let mut rest = line;
    while let Some(stripped) = rest.strip_prefix('#') {
        level += 1;
        rest = stripped;
    }
    let title = rest.trim();
    if level == 0 || title.is_empty() || title.chars().all(|ch| ch == '#' || ch == ' ') {
        return None;
    }
    Some((level.clamp(1, 6), title.to_owned()))
}

/// Fountain's title page is the first block when it is `Key: value` lines.
fn skip_title_page(text: &str) -> &str {
    let trimmed = text.trim_start_matches('\n');
    let first_line = trimmed.lines().next().unwrap_or_default();
    let is_key = first_line.split_once(':').is_some_and(|(key, _)| {
        let key = key.trim();
        !key.is_empty()
            && key.len() <= 32
            && key.chars().all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
    });
    if !is_key {
        return trimmed;
    }
    match trimmed.find("\n\n") {
        Some(end) => &trimmed[end..],
        None => "",
    }
}

fn remove_between(text: &str, open: &str, close: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        output.push_str(&rest[..start]);
        let after = &rest[start + open.len()..];
        match after.find(close) {
            Some(end) => rest = &after[end + close.len()..],
            None => {
                rest = "";
            }
        }
    }
    output.push_str(rest);
    output
}

fn screenplay_blocks_to_prosemirror(blocks: Vec<FountainBlock>) -> Value {
    let mut content: Vec<Value> = Vec::new();
    let mut index = 0;
    while index < blocks.len() {
        match &blocks[index] {
            FountainBlock::Element(..) => {
                let end = (index..blocks.len())
                    .find(|&at| !matches!(blocks[at], FountainBlock::Element(..)))
                    .unwrap_or(blocks.len());
                let elements: Vec<(ScreenplayElement, String)> = blocks[index..end]
                    .iter()
                    .filter_map(|block| match block {
                        FountainBlock::Element(element, text) => Some((*element, text.clone())),
                        _ => None,
                    })
                    .collect();
                for text in force_screenplay_marks(&elements) {
                    content.push(json!({
                        "type": "paragraph",
                        "attrs": { "id": Uuid::now_v7().to_string() },
                        "content": [{ "type": "text", "text": text }]
                    }));
                }
                index = end;
            }
            FountainBlock::Heading(level, title) => {
                content.push(json!({
                    "type": "heading",
                    "attrs": { "id": Uuid::now_v7().to_string(), "level": level },
                    "content": [{ "type": "text", "text": title }]
                }));
                index += 1;
            }
            FountainBlock::Rule => {
                content.push(json!({
                    "type": "horizontalRule",
                    "attrs": { "id": Uuid::now_v7().to_string() }
                }));
                index += 1;
            }
        }
    }
    if content.is_empty() {
        return crate::empty_prosemirror_doc();
    }
    json!({ "type": "doc", "content": content })
}

#[cfg(test)]
mod tests {
    use super::{
        ScreenplayElement::*, classify_screenplay, force_screenplay_marks, fountain_to_prosemirror,
        is_character_cue, is_slugline, is_transition, screenplay_text,
    };
    use serde_json::Value;

    fn texts(doc: &Value) -> Vec<(String, String)> {
        doc["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| {
                let kind = node["type"].as_str().unwrap().to_owned();
                let text = node["content"]
                    .as_array()
                    .map(|runs| {
                        runs.iter()
                            .filter_map(|run| run["text"].as_str())
                            .collect::<String>()
                    })
                    .unwrap_or_default();
                (kind, text)
            })
            .collect()
    }

    #[test]
    fn recognises_sluglines_transitions_and_cues() {
        assert!(is_slugline("INT. KITCHEN - NIGHT"));
        assert!(is_slugline("ext. harbour - dawn"));
        assert!(is_slugline("I/E. CAR - CONTINUOUS"));
        assert!(!is_slugline("INTERIOR MONOLOGUE"));
        assert!(is_transition("CUT TO:"));
        assert!(is_transition("> FADE OUT."));
        assert!(!is_transition("> centred <"));
        assert!(!is_transition("Cut to:"));
        assert!(is_character_cue("MIRA"));
        assert!(is_character_cue("MIRA (V.O.)"));
        assert!(is_character_cue("MIRA ^"));
        assert!(is_character_cue("@McAvoy"));
        assert!(!is_character_cue("BANG."));
        assert!(!is_character_cue("FADE IN:"));
        assert!(!is_character_cue("Mira"));
        assert!(!is_character_cue("ミラ"));
    }

    #[test]
    fn classifies_a_run_of_paragraphs_in_context() {
        let kinds = classify_screenplay(&[
            "FADE IN:",
            "INT. KITCHEN - NIGHT",
            "Mira waits by the stove.",
            "MIRA",
            "(quietly)",
            "I knew you'd come.",
            "(beat)",
            "Sit down.",
            "JO",
            "NO!",
            "!BANG.",
            "The door slams.",
            ".CONTINUOUS",
            "@tom",
            "Hi.",
            "CUT TO:",
            "LATER",
        ]);
        assert_eq!(
            kinds,
            vec![
                Action,
                SceneHeading,
                Action,
                Character,
                Parenthetical,
                Dialogue,
                Parenthetical,
                Dialogue,
                Character,
                Dialogue,
                Action,
                Action,
                SceneHeading,
                Character,
                Dialogue,
                Transition,
                Action,
            ]
        );
    }

    #[test]
    fn strips_forcing_marks_for_formats_that_carry_the_type() {
        assert_eq!(screenplay_text("!BANG.", Action), "BANG.");
        assert_eq!(screenplay_text(".CONTINUOUS", SceneHeading), "CONTINUOUS");
        assert_eq!(screenplay_text("...and then", SceneHeading), "...and then");
        assert_eq!(screenplay_text("@tom ^", Character), "tom");
        assert_eq!(screenplay_text("> FADE OUT.", Transition), "FADE OUT.");
        assert_eq!(screenplay_text("(beat)", Parenthetical), "(beat)");
    }

    #[test]
    fn forces_marks_only_where_the_bare_text_would_be_misread() {
        let forced = force_screenplay_marks(&[
            (Action, "BANG.".into()),
            (Action, "LATER".into()),
            (Action, "The door slams.".into()),
            (Character, "MIRA".into()),
            (Dialogue, "Hello.".into()),
            (Transition, "FADE OUT.".into()),
            (SceneHeading, "CONTINUOUS".into()),
            (Character, "JO".into()),
        ]);
        assert_eq!(
            forced,
            vec![
                "BANG.",
                "!LATER",
                "The door slams.",
                "MIRA",
                "Hello.",
                "> FADE OUT.",
                ".CONTINUOUS",
                "@JO"
            ]
        );
        let refs: Vec<&str> = forced.iter().map(String::as_str).collect();
        assert_eq!(
            classify_screenplay(&refs),
            vec![
                Action,
                Action,
                Action,
                Character,
                Dialogue,
                Transition,
                SceneHeading,
                Character
            ]
        );
    }

    #[test]
    fn imports_fountain_one_paragraph_per_element() {
        let fountain = "Title: Dusk\nCredit: Written by\nAuthor: Someone\n\n# Act One\n\nFADE IN:\n\nINT. KITCHEN - NIGHT\n\nMira waits by the stove.\nThe kettle ticks. /* cut this */\n\nMIRA\n(quietly)\nI knew you'd come.\nSit down.\n(beat)\nPlease. [[tighten]]\n\nBANG.\n\nThe door slams.\n\nCUT TO:\n\n===\n\nEXT. HARBOUR - DAWN\n";
        let doc = fountain_to_prosemirror(fountain);
        assert_eq!(
            texts(&doc),
            vec![
                ("heading".to_owned(), "Act One".to_owned()),
                ("paragraph".to_owned(), "FADE IN:".to_owned()),
                ("paragraph".to_owned(), "INT. KITCHEN - NIGHT".to_owned()),
                (
                    "paragraph".to_owned(),
                    "Mira waits by the stove.\nThe kettle ticks.".to_owned()
                ),
                ("paragraph".to_owned(), "MIRA".to_owned()),
                ("paragraph".to_owned(), "(quietly)".to_owned()),
                (
                    "paragraph".to_owned(),
                    "I knew you'd come.\nSit down.".to_owned()
                ),
                ("paragraph".to_owned(), "(beat)".to_owned()),
                ("paragraph".to_owned(), "Please.".to_owned()),
                ("paragraph".to_owned(), "BANG.".to_owned()),
                ("paragraph".to_owned(), "The door slams.".to_owned()),
                ("paragraph".to_owned(), "CUT TO:".to_owned()),
                ("horizontalRule".to_owned(), String::new()),
                ("paragraph".to_owned(), "EXT. HARBOUR - DAWN".to_owned()),
            ]
        );
        assert_eq!(doc["content"][0]["attrs"]["level"], 1);
    }

    #[test]
    fn import_forces_an_uppercase_action_line_that_would_read_as_a_cue() {
        let doc = fountain_to_prosemirror("LATER\n\nMira sits.\n\nMIRA\nHello.\n");
        let texts: Vec<String> = texts(&doc).into_iter().map(|(_, text)| text).collect();
        assert_eq!(texts, vec!["!LATER", "Mira sits.", "MIRA", "Hello."]);
    }

    #[test]
    fn empty_fountain_is_an_empty_document() {
        let doc = fountain_to_prosemirror("Title: Nothing\n");
        assert_eq!(doc["content"].as_array().unwrap().len(), 1);
        assert_eq!(doc["content"][0]["type"], "paragraph");
    }
}
