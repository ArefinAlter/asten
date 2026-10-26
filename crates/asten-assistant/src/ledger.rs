//! The spend ledger: every provider call is recorded with its tokens and an
//! estimated cost, per thread and per day. Caps are checked before a call and
//! a cap of zero means no model call.

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::{DateTime, Local, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    config::{Caps, Price},
    error::{AssistantError, Result},
};

/// One provider call.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Entry {
    /// When the call finished.
    pub at: DateTime<Utc>,
    /// The run it belongs to.
    pub run_id: String,
    /// The conversation thread.
    pub thread_id: String,
    /// Provider id.
    pub provider: String,
    /// Model name.
    pub model: String,
    /// Tokens sent.
    pub input_tokens: u64,
    /// Tokens received.
    pub output_tokens: u64,
    /// Estimated dollars, when a price is known.
    pub cost_usd: Option<f64>,
    /// True when the provider reported no usage and the counts are estimated.
    #[serde(default)]
    pub estimated: bool,
}

/// What has been spent.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Totals {
    /// Tokens today (local date).
    pub day_tokens: u64,
    /// Input tokens today.
    pub day_input: u64,
    /// Output tokens today.
    pub day_output: u64,
    /// Estimated dollars today.
    pub day_usd: f64,
    /// Calls today.
    pub day_calls: u64,
    /// Tokens in the thread.
    pub thread_tokens: u64,
    /// Estimated dollars in the thread.
    pub thread_usd: f64,
    /// True when some call today had no known price.
    pub day_has_unpriced: bool,
}

/// Dollars for a call at a price.
#[must_use]
pub fn cost_of(price: Option<Price>, input: u64, output: u64) -> Option<f64> {
    price.map(|p| {
        (input as f64) * p.input_per_mtok / 1_000_000.0
            + (output as f64) * p.output_per_mtok / 1_000_000.0
    })
}

/// A rough token count for text, used to check caps before a call.
#[must_use]
pub fn estimate_tokens(text_bytes: usize) -> u64 {
    (text_bytes as u64).div_ceil(4)
}

/// The ledger file and its in-memory copy.
pub struct Ledger {
    path: Option<PathBuf>,
    entries: Mutex<Vec<Entry>>,
}

const KEEP: usize = 20_000;

impl Ledger {
    /// Open (or start) the ledger at `path`.
    #[must_use]
    pub fn open(path: &Path) -> Self {
        let mut entries = Vec::new();
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                if let Ok(entry) = serde_json::from_str::<Entry>(line) {
                    entries.push(entry);
                }
            }
        }
        if entries.len() > KEEP {
            entries.drain(..entries.len() - KEEP);
        }
        Self {
            path: Some(path.to_owned()),
            entries: Mutex::new(entries),
        }
    }

    /// A ledger that is not written to disk.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            path: None,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Record a finished call.
    pub fn record(&self, entry: Entry) -> Result<()> {
        if let Some(path) = &self.path {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let line = serde_json::to_string(&entry)
                .map_err(|error| AssistantError::Io(error.to_string()))?;
            writeln!(file, "{line}")?;
        }
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(entry);
        Ok(())
    }

    /// Totals for today and for one thread.
    #[must_use]
    pub fn totals(&self, thread_id: Option<&str>) -> Totals {
        let today = Local::now().date_naive();
        let entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        let mut totals = Totals::default();
        for entry in entries.iter() {
            if local_day(entry.at) == today {
                totals.day_input += entry.input_tokens;
                totals.day_output += entry.output_tokens;
                totals.day_tokens += entry.input_tokens + entry.output_tokens;
                totals.day_calls += 1;
                match entry.cost_usd {
                    Some(cost) => totals.day_usd += cost,
                    None => totals.day_has_unpriced = true,
                }
            }
            if thread_id == Some(entry.thread_id.as_str()) {
                totals.thread_tokens += entry.input_tokens + entry.output_tokens;
                totals.thread_usd += entry.cost_usd.unwrap_or(0.0);
            }
        }
        totals
    }

    /// The most recent entries, newest first.
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<Entry> {
        let entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
        entries.iter().rev().take(limit).cloned().collect()
    }

    /// Forget everything (the writer asked).
    pub fn clear(&self) -> Result<()> {
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        if let Some(path) = &self.path {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    /// Refuse a call that would break a cap. `planned_tokens` is the estimate
    /// of this call (request plus the most the model may write).
    pub fn check(
        &self,
        caps: &Caps,
        thread_id: &str,
        planned_tokens: u64,
        planned_usd: Option<f64>,
    ) -> Result<()> {
        if caps.daily_tokens == 0 || caps.thread_tokens == 0 {
            return Err(AssistantError::Cap(
                "A spend cap is set to zero, so no model is called. Raise it in Settings → AI providers."
                    .into(),
            ));
        }
        if caps.daily_usd <= 0.0 {
            return Err(AssistantError::Cap(
                "The daily spend cap is zero, so no model is called. Raise it in Settings → AI providers."
                    .into(),
            ));
        }
        let totals = self.totals(Some(thread_id));
        if totals.day_tokens + planned_tokens > caps.daily_tokens {
            return Err(AssistantError::Cap(format!(
                "The daily token cap ({}) would be passed: {} used today. Raise it in Settings → AI providers.",
                caps.daily_tokens, totals.day_tokens
            )));
        }
        if totals.thread_tokens + planned_tokens > caps.thread_tokens {
            return Err(AssistantError::Cap(format!(
                "This thread's token cap ({}) would be passed: {} used. Start a new thread or raise the cap.",
                caps.thread_tokens, totals.thread_tokens
            )));
        }
        if totals.day_usd + planned_usd.unwrap_or(0.0) > caps.daily_usd {
            return Err(AssistantError::Cap(format!(
                "The daily spend cap (${:.2}) would be passed: about ${:.4} used today.",
                caps.daily_usd, totals.day_usd
            )));
        }
        Ok(())
    }
}

fn local_day(at: DateTime<Utc>) -> NaiveDate {
    at.with_timezone(&Local).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(thread: &str, input: u64, output: u64, cost: Option<f64>) -> Entry {
        Entry {
            at: Utc::now(),
            run_id: "r".into(),
            thread_id: thread.into(),
            provider: "anthropic".into(),
            model: "m".into(),
            input_tokens: input,
            output_tokens: output,
            cost_usd: cost,
            estimated: false,
        }
    }

    #[test]
    fn totals_split_by_day_and_thread() {
        let ledger = Ledger::in_memory();
        ledger.record(entry("a", 100, 50, Some(0.01))).unwrap();
        ledger.record(entry("b", 10, 5, None)).unwrap();
        let t = ledger.totals(Some("a"));
        assert_eq!(t.day_tokens, 165);
        assert_eq!(t.thread_tokens, 150);
        assert!(t.day_has_unpriced);
        assert!((t.thread_usd - 0.01).abs() < 1e-9);
    }

    #[test]
    fn a_zero_cap_means_no_call() {
        let ledger = Ledger::in_memory();
        let mut caps = Caps::default();
        assert!(ledger.check(&caps, "t", 100, None).is_ok());
        caps.daily_tokens = 0;
        assert!(ledger.check(&caps, "t", 1, None).is_err());
        caps = Caps::default();
        caps.thread_tokens = 0;
        assert!(ledger.check(&caps, "t", 1, None).is_err());
        caps = Caps::default();
        caps.daily_usd = 0.0;
        assert!(ledger.check(&caps, "t", 1, None).is_err());
    }

    #[test]
    fn caps_count_what_is_planned() {
        let ledger = Ledger::in_memory();
        ledger.record(entry("t", 900, 0, None)).unwrap();
        let caps = Caps {
            daily_tokens: 1000,
            thread_tokens: 1000,
            daily_usd: 5.0,
        };
        assert!(ledger.check(&caps, "t", 50, None).is_ok());
        assert!(ledger.check(&caps, "t", 200, None).is_err());
        assert!(
            ledger.check(&caps, "other", 200, None).is_err(),
            "the day cap is shared"
        );
    }

    #[test]
    fn the_ledger_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ledger.jsonl");
        let ledger = Ledger::open(&path);
        ledger.record(entry("a", 1, 2, Some(0.5))).unwrap();
        let again = Ledger::open(&path);
        assert_eq!(again.totals(Some("a")).thread_tokens, 3);
        again.clear().unwrap();
        assert_eq!(Ledger::open(&path).totals(None).day_tokens, 0);
    }

    #[test]
    fn cost_uses_the_price() {
        let price = Price {
            input_per_mtok: 2.0,
            output_per_mtok: 10.0,
        };
        let cost = cost_of(Some(price), 1_000_000, 100_000).unwrap();
        assert!((cost - 3.0).abs() < 1e-9);
        assert!(cost_of(None, 1, 1).is_none());
    }
}
