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
    value
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

#[must_use]
pub fn prosemirror_to_fountain(title: &str, value: &Value) -> String {
    let body = prosemirror_to_markdown(value);
    format!("Title: {title}\n\n{body}")
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
