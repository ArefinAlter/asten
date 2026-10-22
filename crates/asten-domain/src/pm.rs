use serde_json::{Value, json};
use uuid::Uuid;

#[must_use]
pub fn empty_prosemirror_doc() -> Value {
    json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "attrs": { "id": Uuid::now_v7().to_string() },
            "content": []
        }]
    })
}

#[must_use]
pub fn ensure_block_ids(mut value: Value) -> Value {
    assign_ids(&mut value);
    dedupe_top_level_ids(&mut value);
    value
}

/// The block model keys top-level nodes by id. A paste that carried ids along
/// could otherwise make one block silently replace another; the first keeps
/// its id and later twins are re-minted.
fn dedupe_top_level_ids(value: &mut Value) {
    let Some(content) = value.get_mut("content").and_then(Value::as_array_mut) else {
        return;
    };
    let mut seen = std::collections::HashSet::new();
    for node in content {
        let Some(attrs) = node.get_mut("attrs").and_then(Value::as_object_mut) else {
            continue;
        };
        let Some(id) = attrs.get("id").and_then(Value::as_str).map(str::to_owned) else {
            continue;
        };
        if !seen.insert(id) {
            attrs.insert("id".into(), json!(Uuid::now_v7().to_string()));
        }
    }
}

fn assign_ids(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                assign_ids(item);
            }
        }
        Value::Object(map) => {
            let node_type = map
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if !node_type.is_empty() && node_type != "text" {
                let attrs = map.entry("attrs").or_insert_with(|| json!({}));
                if let Some(attrs) = attrs.as_object_mut() {
                    let missing = attrs
                        .get("id")
                        .and_then(Value::as_str)
                        .map(|id| id.is_empty())
                        .unwrap_or(true);
                    if missing {
                        attrs.insert("id".into(), json!(Uuid::now_v7().to_string()));
                    }
                }
            }
            if let Some(content) = map.get_mut("content") {
                assign_ids(content);
            }
        }
        _ => {}
    }
}

#[must_use]
pub fn prosemirror_to_markdown(value: &Value) -> String {
    let mut output = String::new();
    write_markdown(value, &mut output, 0);
    while output.ends_with('\n') {
        output.pop();
    }
    output.push('\n');
    output
}

fn write_markdown(value: &Value, output: &mut String, list_depth: usize) {
    let Some(node_type) = value.get("type").and_then(Value::as_str) else {
        if let Some(text) = value.get("text").and_then(Value::as_str) {
            output.push_str(text);
        }
        if let Some(content) = value.get("content") {
            write_markdown(content, output, list_depth);
        }
        return;
    };
    match node_type {
        "doc" => {
            if let Some(content) = value.get("content").and_then(Value::as_array) {
                for child in content {
                    write_markdown(child, output, list_depth);
                }
            }
        }
        "heading" => {
            let level = value
                .pointer("/attrs/level")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 6) as usize;
            output.push_str(&"#".repeat(level));
            output.push(' ');
            write_inline(value, output);
            output.push_str("\n\n");
        }
        "paragraph" => {
            write_inline(value, output);
            output.push_str("\n\n");
        }
        "blockquote" => {
            let mut inner = String::new();
            if let Some(content) = value.get("content").and_then(Value::as_array) {
                for child in content {
                    write_markdown(child, &mut inner, list_depth);
                }
            }
            for line in inner.trim_end().lines() {
                output.push_str("> ");
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
        "bulletList" => write_list(value, output, list_depth, false),
        "orderedList" => write_list(value, output, list_depth, true),
        "horizontalRule" => output.push_str("* * *\n\n"),
        "sceneBreak" => output.push_str("# # #\n\n"),
        "text" => write_inline(value, output),
        _ => {
            if let Some(content) = value.get("content") {
                write_markdown(content, output, list_depth);
            }
        }
    }
}

fn write_list(value: &Value, output: &mut String, depth: usize, ordered: bool) {
    let Some(items) = value.get("content").and_then(Value::as_array) else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        output.push_str(&"  ".repeat(depth));
        if ordered {
            output.push_str(&(index + 1).to_string());
            output.push_str(". ");
        } else {
            output.push_str("- ");
        }
        let mut inner = String::new();
        write_markdown(item, &mut inner, depth + 1);
        output.push_str(inner.trim());
        output.push('\n');
    }
    output.push('\n');
}

fn write_inline(value: &Value, output: &mut String) {
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        output.push_str(&apply_marks(text, value.get("marks")));
        return;
    }
    if let Some(content) = value.get("content").and_then(Value::as_array) {
        for child in content {
            write_inline(child, output);
        }
    }
}

fn apply_marks(text: &str, marks: Option<&Value>) -> String {
    let mut wrapped = text.to_owned();
    let Some(marks) = marks.and_then(Value::as_array) else {
        return wrapped;
    };
    for mark in marks {
        let Some(kind) = mark.get("type").and_then(Value::as_str) else {
            continue;
        };
        wrapped = match kind {
            "bold" | "strong" => format!("**{wrapped}**"),
            "italic" | "em" => format!("*{wrapped}*"),
            "underline" => format!("<u>{wrapped}</u>"),
            "strike" | "deletion" => format!("~~{wrapped}~~"),
            "insertion" => format!("++{wrapped}++"),
            "code" => format!("`{wrapped}`"),
            _ => wrapped,
        };
    }
    wrapped
}

/// Fountain for one document. A slugline title is a scene heading; any other
/// title is a Fountain section (`# Title`), which readers index but do not
/// print. Paragraphs are classified together so that a cue, its
/// parentheticals and its speech stay on consecutive lines, which is how
/// Fountain tells dialogue from action.
#[must_use]
pub fn prosemirror_to_fountain(title: &str, value: &Value) -> String {
    use crate::screenplay::{ScreenplayElement, classify_screenplay, is_slugline};

    let mut output = String::new();
    let title = title.trim();
    if !title.is_empty() {
        if !is_slugline(title) {
            output.push_str("# ");
        }
        output.push_str(title);
        output.push_str("\n\n");
    }
    let empty = Vec::new();
    let blocks = value
        .get("content")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let block_type = |block: &Value| {
        block
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let mut index = 0;
    while index < blocks.len() {
        if block_type(&blocks[index]) != "paragraph" {
            write_markdown(&blocks[index], &mut output, 0);
            index += 1;
            continue;
        }
        let end = (index..blocks.len())
            .find(|&at| block_type(&blocks[at]) != "paragraph")
            .unwrap_or(blocks.len());
        let texts: Vec<String> = blocks[index..end]
            .iter()
            .map(|block| {
                let mut text = String::new();
                write_inline(block, &mut text);
                text.trim().to_owned()
            })
            .filter(|text| !text.is_empty())
            .collect();
        let kinds = classify_screenplay(&texts.iter().map(String::as_str).collect::<Vec<_>>());
        for (at, text) in texts.iter().enumerate() {
            output.push_str(text);
            let speech_continues = matches!(
                kinds[at],
                ScreenplayElement::Character
                    | ScreenplayElement::Parenthetical
                    | ScreenplayElement::Dialogue
            ) && matches!(
                kinds.get(at + 1),
                Some(ScreenplayElement::Dialogue | ScreenplayElement::Parenthetical)
            );
            output.push_str(if speech_continues { "\n" } else { "\n\n" });
        }
        index = end;
    }
    while output.ends_with('\n') {
        output.pop();
    }
    output.push('\n');
    output
}

#[must_use]
pub fn markdown_to_prosemirror(markdown: &str) -> Value {
    let mut content = Vec::new();
    for block in markdown.replace('\r', "").split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "* * *" || trimmed == "---" {
            content.push(json!({
                "type": "horizontalRule",
                "attrs": { "id": Uuid::now_v7().to_string() }
            }));
            continue;
        }
        if trimmed == "# # #" {
            content.push(json!({
                "type": "sceneBreak",
                "attrs": { "id": Uuid::now_v7().to_string() }
            }));
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix('#') {
            let mut level = 1u32;
            let mut rest = heading;
            while let Some(stripped) = rest.strip_prefix('#') {
                level += 1;
                rest = stripped;
            }
            let text = rest.trim();
            content.push(json!({
                "type": "heading",
                "attrs": { "id": Uuid::now_v7().to_string(), "level": level.clamp(1, 6) },
                "content": [{ "type": "text", "text": text }]
            }));
            continue;
        }
        content.push(json!({
            "type": "paragraph",
            "attrs": { "id": Uuid::now_v7().to_string() },
            "content": [{ "type": "text", "text": trimmed }]
        }));
    }
    if content.is_empty() {
        return empty_prosemirror_doc();
    }
    json!({ "type": "doc", "content": content })
}

#[must_use]
pub fn plaintext_to_prosemirror(text: &str) -> Value {
    markdown_to_prosemirror(text)
}
