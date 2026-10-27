//! A small incremental Server-Sent Events parser.
//!
//! Providers stream `text/event-stream`: lines of `field: value`, events
//! separated by a blank line. This parser takes bytes as they arrive, copes
//! with CRLF, a split UTF-8 character, comments and multi-line `data`, and
//! refuses a line or an event that grows without bound.

/// One event from the stream.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SseEvent {
    /// The `event:` field, if any.
    pub event: Option<String>,
    /// The `data:` lines joined with newlines.
    pub data: String,
}

/// Longest single line or event accepted.
pub const MAX_EVENT_BYTES: usize = 4 * 1024 * 1024;

/// The parser's state between chunks.
#[derive(Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    event: Option<String>,
    data: Vec<String>,
    data_bytes: usize,
    saw_field: bool,
}

/// The stream was malformed or too large.
#[derive(Debug, Eq, PartialEq)]
pub struct SseError(pub &'static str);

impl SseParser {
    /// A new parser.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed bytes; returns the events they completed.
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<SseEvent>, SseError> {
        self.buffer.extend_from_slice(bytes);
        let mut events = Vec::new();
        loop {
            let Some(end) = self.buffer.iter().position(|&b| b == b'\n') else {
                if self.buffer.len() > MAX_EVENT_BYTES {
                    return Err(SseError("a line in the stream was too long"));
                }
                break;
            };
            let mut line: Vec<u8> = self.buffer.drain(..=end).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let line = String::from_utf8_lossy(&line).into_owned();
            if let Some(event) = self.line(&line)? {
                events.push(event);
            }
        }
        Ok(events)
    }

    /// The stream ended: a final event without a trailing blank line still counts.
    pub fn finish(&mut self) -> Option<SseEvent> {
        if !self.buffer.is_empty() {
            let line = String::from_utf8_lossy(&std::mem::take(&mut self.buffer)).into_owned();
            let _ = self.line(line.trim_end_matches(['\r', '\n']));
        }
        self.dispatch()
    }

    fn line(&mut self, line: &str) -> Result<Option<SseEvent>, SseError> {
        if line.is_empty() {
            return Ok(self.dispatch());
        }
        if line.starts_with(':') {
            return Ok(None);
        }
        let (field, value) = match line.split_once(':') {
            Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
            None => (line, ""),
        };
        match field {
            "event" => {
                self.event = Some(value.to_owned());
                self.saw_field = true;
            }
            "data" => {
                self.data_bytes += value.len() + 1;
                if self.data_bytes > MAX_EVENT_BYTES {
                    return Err(SseError("an event in the stream was too large"));
                }
                self.data.push(value.to_owned());
                self.saw_field = true;
            }
            _ => {}
        }
        Ok(None)
    }

    fn dispatch(&mut self) -> Option<SseEvent> {
        if !self.saw_field {
            return None;
        }
        self.saw_field = false;
        self.data_bytes = 0;
        let event = SseEvent {
            event: self.event.take(),
            data: std::mem::take(&mut self.data).join("\n"),
        };
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_split_across_chunks() {
        let mut parser = SseParser::new();
        assert!(parser.push(b"event: message_start\nda").unwrap().is_empty());
        let events = parser.push(b"ta: {\"a\":1}\n\ndata: two\r\n\r\n").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("message_start"));
        assert_eq!(events[0].data, "{\"a\":1}");
        assert_eq!(events[1].data, "two");
    }

    #[test]
    fn comments_and_multiline_data() {
        let mut parser = SseParser::new();
        let events = parser.push(b": ping\n\ndata: a\ndata: b\n\n").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "a\nb");
    }

    #[test]
    fn a_split_multibyte_character_survives() {
        let text = "data: caf\u{e9}\n\n".as_bytes().to_vec();
        let mut parser = SseParser::new();
        let cut = text.iter().position(|&b| b == 0xc3).unwrap() + 1;
        assert!(parser.push(&text[..cut]).unwrap().is_empty());
        let events = parser.push(&text[cut..]).unwrap();
        assert_eq!(events[0].data, "caf\u{e9}");
    }

    #[test]
    fn the_last_event_without_a_blank_line_is_kept() {
        let mut parser = SseParser::new();
        assert!(parser.push(b"data: [DONE]").unwrap().is_empty());
        assert_eq!(parser.finish().unwrap().data, "[DONE]");
    }

    #[test]
    fn an_endless_line_is_refused() {
        let mut parser = SseParser::new();
        let big = vec![b'x'; MAX_EVENT_BYTES + 10];
        assert!(parser.push(&big).is_err());
    }
}
