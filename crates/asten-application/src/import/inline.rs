//! Inline Markdown: `*italic*`, `_italic_`, `**bold**`, `__bold__`, `~~struck~~`,
//! `++inserted++` (Asten's own export of a tracked insertion), `` `code` ``,
//! `[text](url)` (the text is kept), backslash escapes.
//!
//! A delimiter opens only when the next character is not white space, closes
//! only when the previous one is not, and an underscore never opens or closes
//! inside a word (`snake_case` stays as it is). An unmatched delimiter is just
//! a character. The search for a closer is bounded.

use super::blocks::{Inline, marked, tidy};

const ESCAPABLE: &str = "\\`*_{}[]()#+-.!~>|<";
const STEPS: usize = 400_000;

/// Parse one paragraph's text into spans.
pub(crate) fn parse(text: &str) -> Vec<Inline> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut budget = STEPS;
    run(&chars, &mut Vec::new(), &mut out, &mut budget);
    tidy(out)
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric()
}

fn run(chars: &[char], marks: &mut Vec<&'static str>, out: &mut Vec<Inline>, budget: &mut usize) {
    let mut literal = String::new();
    let flush = |literal: &mut String, marks: &[&'static str], out: &mut Vec<Inline>| {
        if !literal.is_empty() {
            out.push(marked(literal, marks));
            literal.clear();
        }
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() && ESCAPABLE.contains(chars[i + 1]) {
            literal.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '`' {
            let run_len = chars[i..].iter().take_while(|&&x| x == '`').count();
            if let Some(end) = find_run(chars, i + run_len, '`', run_len, budget) {
                flush(&mut literal, marks, out);
                let inner: String = chars[i + run_len..end].iter().collect();
                let mut with_code = marks.clone();
                with_code.push("code");
                out.push(marked(inner.trim(), &with_code));
                i = end + run_len;
                continue;
            }
        }
        if c == '!' && chars.get(i + 1) == Some(&'[') {
            if let Some((_, after)) = link(chars, i + 1, budget) {
                // An image: the page has no pictures, so nothing is kept.
                i = after;
                continue;
            }
        }
        if c == '[' {
            if let Some(((from, to), after)) = link(chars, i, budget) {
                flush(&mut literal, marks, out);
                run(&chars[from..to], marks, out, budget);
                i = after;
                continue;
            }
        }
        if matches!(c, '*' | '_' | '~' | '+') {
            let double = chars.get(i + 1) == Some(&c) && matches!(c, '*' | '_' | '~' | '+');
            let tries: &[usize] = if matches!(c, '~' | '+') {
                &[2]
            } else if double {
                &[2, 1]
            } else {
                &[1]
            };
            let mut matched = false;
            for &len in tries {
                let mark = match (c, len) {
                    ('*' | '_', 2) => "bold",
                    ('*' | '_', 1) => "italic",
                    ('~', 2) => "strike",
                    ('+', 2) => "insertion",
                    _ => continue,
                };
                if let Some(close) = closer(chars, i, c, len, budget) {
                    flush(&mut literal, marks, out);
                    marks.push(mark);
                    run(&chars[i + len..close], marks, out, budget);
                    marks.pop();
                    i = close + len;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }
        literal.push(c);
        i += 1;
    }
    flush(&mut literal, marks, out);
}

/// Find the run of `len` copies of `ch` that closes a code span.
fn find_run(
    chars: &[char],
    from: usize,
    ch: char,
    len: usize,
    budget: &mut usize,
) -> Option<usize> {
    let mut k = from;
    while k < chars.len() {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        if chars[k] == ch {
            let n = chars[k..].iter().take_while(|&&x| x == ch).count();
            if n == len {
                return Some(k);
            }
            k += n;
        } else {
            k += 1;
        }
    }
    None
}

/// `[text](url)` starting at `[`: the range of the text and the index after `)`.
fn link(chars: &[char], at: usize, budget: &mut usize) -> Option<((usize, usize), usize)> {
    let mut depth = 0usize;
    let mut k = at;
    while k < chars.len() {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        match chars[k] {
            '\\' => k += 1,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    if chars.get(k + 1) == Some(&'(') {
                        let mut j = k + 2;
                        while j < chars.len() && chars[j] != ')' && chars[j] != '\n' {
                            j += 1;
                        }
                        if chars.get(j) == Some(&')') {
                            return Some(((at + 1, k), j + 1));
                        }
                    }
                    return None;
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
}

/// Where the delimiter opened at `open` closes, if it does.
fn closer(chars: &[char], open: usize, ch: char, len: usize, budget: &mut usize) -> Option<usize> {
    let after = open + len;
    let first = *chars.get(after)?;
    if first.is_whitespace() {
        return None;
    }
    if ch == '_' && open > 0 && is_word(chars[open - 1]) {
        return None;
    }
    // `*` right before `*` of a longer run is not this delimiter's opener.
    if len == 1 && chars.get(after) == Some(&ch) {
        return None;
    }
    let mut k = after;
    while k < chars.len() {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        match chars[k] {
            '\\' => {
                k += 2;
                continue;
            }
            '`' => {
                let n = chars[k..].iter().take_while(|&&x| x == '`').count();
                if let Some(end) = find_run(chars, k + n, '`', n, budget) {
                    k = end + n;
                    continue;
                }
                k += n;
                continue;
            }
            c if c == ch => {
                let n = chars[k..].iter().take_while(|&&x| x == ch).count();
                let prev = chars[k - 1];
                let next = chars.get(k + n).copied();
                if n == len && k > after && !prev.is_whitespace() {
                    let in_word = ch == '_' && next.is_some_and(is_word);
                    if !in_word {
                        return Some(k);
                    }
                }
                k += n;
                continue;
            }
            _ => {}
        }
        k += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::blocks::inline_text;

    fn flat(inlines: &[Inline]) -> Vec<(String, Vec<&'static str>)> {
        inlines
            .iter()
            .filter_map(|i| match i {
                Inline::Text(s) => Some((s.text.clone(), s.marks.clone())),
                Inline::Break => None,
            })
            .collect()
    }

    #[test]
    fn emphasis_in_the_forms_writers_use() {
        assert_eq!(
            flat(&parse("She said _never_ and *twice* and **loudly**.")),
            vec![
                ("She said ".into(), vec![]),
                ("never".into(), vec!["italic"]),
                (" and ".into(), vec![]),
                ("twice".into(), vec!["italic"]),
                (" and ".into(), vec![]),
                ("loudly".into(), vec!["bold"]),
                (".".into(), vec![]),
            ]
        );
    }

    #[test]
    fn nesting_and_the_tracked_forms() {
        assert_eq!(
            flat(&parse("**bold and _italic_** ~~gone~~ ++added++")),
            vec![
                ("bold and ".into(), vec!["bold"]),
                ("italic".into(), vec!["bold", "italic"]),
                (" ".into(), vec![]),
                ("gone".into(), vec!["strike"]),
                (" ".into(), vec![]),
                ("added".into(), vec!["insertion"]),
            ]
        );
    }

    #[test]
    fn words_with_underscores_and_stray_stars_stay_as_they_are() {
        assert_eq!(
            inline_text(&parse("see snake_case_name and a * b * c")),
            "see snake_case_name and a * b * c"
        );
        assert_eq!(
            flat(&parse("2 * 3 = 6")),
            vec![("2 * 3 = 6".into(), vec![])]
        );
        assert_eq!(
            flat(&parse("an unclosed *star")),
            vec![("an unclosed *star".into(), vec![])]
        );
    }

    #[test]
    fn escapes_code_and_links() {
        assert_eq!(
            inline_text(&parse(r"a \*literal\* star")),
            "a *literal* star"
        );
        assert_eq!(
            flat(&parse("run `a_b *c*` now")),
            vec![
                ("run ".into(), vec![]),
                ("a_b *c*".into(), vec!["code"]),
                (" now".into(), vec![]),
            ]
        );
        assert_eq!(
            inline_text(&parse("see [the key](https://example.org/key) here")),
            "see the key here"
        );
        assert_eq!(inline_text(&parse("a ![picture](x.png) b")), "a  b");
    }

    #[test]
    fn it_is_bounded() {
        let long = "*a ".repeat(20_000);
        let parsed = parse(&long);
        assert!(!parsed.is_empty());
    }
}
