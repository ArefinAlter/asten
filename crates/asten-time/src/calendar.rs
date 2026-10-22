//! Calendars and story-time labels.
//!
//! A project chooses a base unit (day, hour, or minute) and a calendar. Labels
//! the author types ("day 14", "412-03-01", "three days after the siege",
//! "that winter") become tick bounds or relative constraints. Fuzzy labels become
//! intervals, never fake precision. Numerals in any script are read through
//! their Unicode decimal-digit blocks, so the parser does not assume ASCII.

use asten_ir::{Tick, TimeBounds};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Minute,
    Hour,
    #[default]
    Day,
}

impl TimeUnit {
    #[must_use]
    pub const fn ticks_per_day(self) -> Tick {
        match self {
            TimeUnit::Minute => 1440,
            TimeUnit::Hour => 24,
            TimeUnit::Day => 1,
        }
    }
}

/// A month in a fictional calendar.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
pub struct Month {
    pub name: String,
    pub days: u32,
}

/// Every `every` years, `extra_days` are added to month `month` (zero-based).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
pub struct LeapRule {
    pub every: u32,
    pub extra_days: u32,
    pub month: usize,
}

impl FictionalCalendar {
    /// Reject calendars the arithmetic cannot handle: a month of zero days, a
    /// leap rule pointing past the last month, an absurd number of months.
    pub fn validate(&self) -> Result<(), String> {
        if self.months.len() > 64 {
            return Err("a calendar may have at most 64 months".into());
        }
        if let Some(month) = self.months.iter().find(|month| month.days == 0) {
            return Err(format!("the month {:?} has no days", month.name));
        }
        if self.months.iter().any(|month| month.days > 100_000) {
            return Err("a month may have at most 100000 days".into());
        }
        if let Some(rule) = &self.leap {
            if rule.every == 0 {
                return Err("a leap rule needs `every` of at least 1".into());
            }
            if rule.month >= self.months.len() {
                return Err("the leap rule points past the last month".into());
            }
            if rule.extra_days > 100_000 {
                return Err("a leap rule may add at most 100000 days".into());
            }
        }
        if self.epoch_year.abs() > 1_000_000_000 {
            return Err("the epoch year is out of range".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
pub struct FictionalCalendar {
    pub months: Vec<Month>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<LeapRule>,
    /// The year that is day 0.
    #[serde(default)]
    pub epoch_year: i64,
    /// Era suffixes the author may write after a year ("412 AE").
    #[serde(default)]
    pub eras: Vec<String>,
    /// Season names in order, each spanning an equal share of the year.
    #[serde(default)]
    pub seasons: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Calendar {
    /// Days counted from an unnamed day 0. "Day 14" is tick 14 × ticks per day.
    #[default]
    DayCount,
    /// The proleptic Gregorian calendar; ISO dates. Tick 0 is 0000-03-01.
    Gregorian,
    Fictional(FictionalCalendar),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
pub struct Clock {
    #[serde(default)]
    pub unit: TimeUnit,
    #[serde(default)]
    pub calendar: Calendar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Granularity {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Season,
    Year,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "time/"))]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    After,
    Before,
    /// "that night", "the same day": within the same day as the anchor.
    Within,
    /// "meanwhile": the same time as the anchor.
    Same,
}

/// What a label meant.
#[derive(Clone, Debug, PartialEq)]
pub enum Parsed {
    /// Pinned in the calendar, possibly to an interval.
    Absolute(TimeBounds),
    /// Relative to another time point. `anchor` is the author's phrase for it,
    /// or `None` for "the previous scene". Amounts are in ticks.
    Relative {
        relation: Relation,
        lo: Tick,
        hi: Tick,
        anchor: Option<String>,
    },
}

impl Clock {
    #[must_use]
    pub fn new(unit: TimeUnit, calendar: Calendar) -> Self {
        Self { unit, calendar }
    }

    #[must_use]
    pub const fn ticks_per_day(&self) -> Tick {
        self.unit.ticks_per_day()
    }

    /// Parse a story-time label. Returns `None` when nothing time-like was found.
    #[must_use]
    pub fn parse(&self, label: &str) -> Option<Parsed> {
        let text = normalise(label);
        if text.is_empty() {
            return None;
        }
        if let Some(relative) = self.parse_relative(&text) {
            return Some(relative);
        }
        let (fuzzy, core) = strip_fuzzy(&text);
        let mut bounds = self.parse_absolute(&core)?;
        if fuzzy {
            bounds = self.widen(bounds);
        }
        Some(Parsed::Absolute(bounds))
    }

    /// Format a tick as a label in this calendar.
    #[must_use]
    pub fn format(&self, tick: Tick) -> String {
        let per_day = self.ticks_per_day();
        let day = tick.div_euclid(per_day);
        let rest = tick.rem_euclid(per_day);
        let date = match &self.calendar {
            Calendar::DayCount => format!("Day {day}"),
            Calendar::Gregorian => {
                let (year, month, dom) = civil_from_days(day);
                format!("{year:04}-{month:02}-{dom:02}")
            }
            Calendar::Fictional(calendar) => {
                let (year, month_index, dom) = fictional_from_days(calendar, day);
                let month = calendar
                    .months
                    .get(month_index)
                    .map(|month| month.name.clone())
                    .unwrap_or_else(|| format!("month {}", month_index + 1));
                format!("{dom} {month} {year}")
            }
        };
        match self.unit {
            TimeUnit::Day => date,
            TimeUnit::Hour => format!("{date} {rest:02}:00"),
            TimeUnit::Minute => format!("{date} {:02}:{:02}", rest / 60, rest % 60),
        }
    }

    /// Format bounds as a label.
    #[must_use]
    pub fn format_bounds(&self, bounds: &TimeBounds) -> String {
        match (bounds.lo, bounds.hi) {
            (Some(lo), Some(hi)) if lo == hi => self.format(lo),
            (Some(lo), Some(hi)) => format!("{} to {}", self.format(lo), self.format(hi)),
            (Some(lo), None) => format!("from {}", self.format(lo)),
            (None, Some(hi)) => format!("until {}", self.format(hi)),
            (None, None) => "unplaced".into(),
        }
    }

    fn widen(&self, bounds: TimeBounds) -> TimeBounds {
        let (Some(lo), Some(hi)) = (bounds.lo, bounds.hi) else {
            return bounds;
        };
        let span = hi - lo + 1;
        let per_day = self.ticks_per_day();
        // "About day 10" gives ±3 days; "around 1890" gives ±10% of the span, at least a year.
        let pad = if span <= per_day {
            3 * per_day
        } else {
            (span / 10).max(span.min(366 * per_day))
        };
        TimeBounds::between(lo - pad, hi + pad)
    }

    fn parse_absolute(&self, text: &str) -> Option<TimeBounds> {
        let per_day = self.ticks_per_day();
        let (text, clock_time) = split_clock_time(text);
        let (text, time_of_day) = split_time_of_day(&text);
        let text = text.trim_matches(|c: char| c == ',' || c == ' ');

        let day_bounds = match &self.calendar {
            Calendar::DayCount => parse_day_count(text),
            Calendar::Gregorian => parse_gregorian(text),
            Calendar::Fictional(calendar) => parse_fictional(calendar, text),
        }?;

        let lo = day_bounds.0 * per_day;
        let hi = (day_bounds.1 + 1) * per_day - 1;
        let mut bounds = TimeBounds::between(lo, hi.max(lo));
        if day_bounds.0 == day_bounds.1 && self.unit != TimeUnit::Day {
            if let Some((hour, minute)) = clock_time {
                // "17:30" as `format` writes it: one tick, at the clock's resolution.
                let per_hour = per_day / 24;
                let per_minute = per_day / 1440;
                let start = lo + Tick::from(hour) * per_hour + Tick::from(minute) * per_minute;
                bounds = TimeBounds::at(start.clamp(lo, hi.max(lo)));
            } else if let Some((from_hour, to_hour)) = time_of_day {
                let per_hour = per_day / 24;
                let start = lo + Tick::from(from_hour) * per_hour;
                let end = lo + Tick::from(to_hour) * per_hour - 1;
                bounds = TimeBounds::between(start, end.max(start));
            }
        }
        Some(bounds)
    }

    fn parse_relative(&self, text: &str) -> Option<Parsed> {
        let per_day = self.ticks_per_day();
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }

        // Fixed phrases first.
        let phrase = words.join(" ");
        let fixed: &[(&str, Relation, Tick, Tick)] = &[
            ("meanwhile", Relation::Same, 0, 0),
            ("at the same time", Relation::Same, 0, 0),
            ("same time", Relation::Same, 0, 0),
            ("that night", Relation::Within, 0, per_day - 1),
            ("that evening", Relation::Within, 0, per_day - 1),
            ("that morning", Relation::Within, 0, per_day - 1),
            ("that afternoon", Relation::Within, 0, per_day - 1),
            ("same day", Relation::Within, 0, per_day - 1),
            ("the same day", Relation::Within, 0, per_day - 1),
            ("later that day", Relation::Within, 0, per_day - 1),
            ("later that night", Relation::Within, 0, per_day - 1),
            ("later", Relation::After, 1, 7 * per_day),
            ("soon after", Relation::After, 1, 3 * per_day),
            ("shortly after", Relation::After, 1, 2 * per_day),
            ("the next day", Relation::After, per_day, per_day),
            ("next day", Relation::After, per_day, per_day),
            ("the next morning", Relation::After, per_day, per_day),
            ("the next night", Relation::After, per_day, per_day),
            ("the following day", Relation::After, per_day, per_day),
            ("the following morning", Relation::After, per_day, per_day),
            ("the day after", Relation::After, per_day, per_day),
            ("the day before", Relation::Before, per_day, per_day),
            ("the previous day", Relation::Before, per_day, per_day),
            ("the next week", Relation::After, 7 * per_day, 7 * per_day),
            (
                "the following week",
                Relation::After,
                7 * per_day,
                7 * per_day,
            ),
            (
                "the week before",
                Relation::Before,
                7 * per_day,
                7 * per_day,
            ),
            (
                "the next year",
                Relation::After,
                365 * per_day,
                365 * per_day,
            ),
            (
                "the following year",
                Relation::After,
                365 * per_day,
                365 * per_day,
            ),
            (
                "the year before",
                Relation::Before,
                365 * per_day,
                365 * per_day,
            ),
            ("earlier", Relation::Before, 1, 30 * per_day),
            ("some time later", Relation::After, per_day, 90 * per_day),
            ("much later", Relation::After, 30 * per_day, 3650 * per_day),
            ("long ago", Relation::Before, 365 * per_day, 36500 * per_day),
            (
                "years earlier",
                Relation::Before,
                365 * per_day,
                36500 * per_day,
            ),
            (
                "years later",
                Relation::After,
                365 * per_day,
                36500 * per_day,
            ),
            (
                "months earlier",
                Relation::Before,
                30 * per_day,
                365 * per_day,
            ),
            ("months later", Relation::After, 30 * per_day, 365 * per_day),
            ("weeks later", Relation::After, 7 * per_day, 60 * per_day),
            ("weeks earlier", Relation::Before, 7 * per_day, 60 * per_day),
            ("days later", Relation::After, per_day, 14 * per_day),
            ("days earlier", Relation::Before, per_day, 14 * per_day),
            (
                "hours later",
                Relation::After,
                (per_day / 24).max(1),
                per_day,
            ),
        ];
        for (needle, relation, lo, hi) in fixed {
            if phrase == *needle {
                return Some(Parsed::Relative {
                    relation: *relation,
                    lo: *lo,
                    hi: *hi,
                    anchor: None,
                });
            }
        }

        // "<amount> <unit>(s) <direction> [the] <anchor>" and "<amount> <unit> later".
        let (amount_lo, amount_hi, consumed) = parse_amount(&words)?;
        let unit_word = words.get(consumed)?;
        let (unit_minutes, _) = unit_minutes(unit_word)?;
        let direction = words.get(consumed + 1)?;
        let (relation, anchor_from) = match *direction {
            "after" | "since" | "following" | "past" | "from" | "into" => {
                (Relation::After, consumed + 2)
            }
            "later" | "on" | "afterwards" | "afterward" | "hence" => {
                (Relation::After, consumed + 2)
            }
            "before" | "earlier" | "prior" | "ago" | "previously" | "back" => {
                (Relation::Before, consumed + 2)
            }
            _ => return None,
        };
        let mut anchor_words: Vec<&str> = words
            .get(anchor_from..)
            .map(|slice| slice.to_vec())
            .unwrap_or_default();
        if anchor_words.first().copied() == Some("to") {
            anchor_words.remove(0);
        }
        if anchor_words.first().copied() == Some("the") {
            anchor_words.remove(0);
        }
        let anchor = if anchor_words.is_empty() {
            None
        } else {
            Some(anchor_words.join(" "))
        };
        // Amounts are exact in minutes; at a coarser tick they round outwards,
        // so "three hours" on a day clock is zero to one day, not three days.
        Some(Parsed::Relative {
            relation,
            lo: ticks_from_minutes(amount_lo.saturating_mul(unit_minutes), per_day, false),
            hi: ticks_from_minutes(amount_hi.saturating_mul(unit_minutes), per_day, true),
            anchor,
        })
    }
}

// ───────────────────────── label helpers ─────────────────────────

fn normalise(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for ch in label.trim().chars() {
        if let Some(digit) = decimal_digit(ch) {
            out.push(char::from(b'0' + digit as u8));
        } else if ch == '–' || ch == '—' {
            out.push('-');
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Map a Unicode decimal digit (Nd) from the common scripts to its value.
fn decimal_digit(ch: char) -> Option<u32> {
    if let Some(value) = ch.to_digit(10) {
        return Some(value);
    }
    const ZEROS: &[u32] = &[
        0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
        0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0,
        0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0,
        0xAA50, 0xABF0, 0xFF10, 0x11136,
    ];
    let code = u32::from(ch);
    ZEROS
        .iter()
        .find(|zero| code >= **zero && code < **zero + 10)
        .map(|zero| code - zero)
}

fn strip_fuzzy(text: &str) -> (bool, String) {
    let prefixes = [
        "about ",
        "around ",
        "circa ",
        "c. ",
        "ca. ",
        "roughly ",
        "approximately ",
        "~",
        "some time in ",
        "sometime in ",
    ];
    for prefix in prefixes {
        if let Some(rest) = text.strip_prefix(prefix) {
            return (true, rest.trim().to_owned());
        }
    }
    let suffixes = [" or so", " or thereabouts", "ish"];
    for suffix in suffixes {
        if let Some(rest) = text.strip_suffix(suffix) {
            return (true, rest.trim().to_owned());
        }
    }
    (false, text.to_owned())
}

/// Split a trailing or leading time-of-day word and return hour bounds for it.
fn split_time_of_day(text: &str) -> (String, Option<(u32, u32)>) {
    let table: &[(&str, (u32, u32))] = &[
        ("midnight", (0, 1)),
        ("dawn", (5, 7)),
        ("sunrise", (5, 7)),
        ("morning", (6, 12)),
        ("noon", (12, 13)),
        ("midday", (12, 13)),
        ("afternoon", (12, 17)),
        ("dusk", (17, 20)),
        ("sunset", (17, 20)),
        ("evening", (17, 22)),
        ("night", (20, 24)),
    ];
    let mut words: Vec<&str> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .collect();
    let mut hours = None;
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        if let Some((_, span)) = table.iter().find(|(name, _)| *name == word) {
            hours = Some(*span);
            words.remove(index);
            // Drop a dangling "at" / "in the" / "of" left beside the word.
            while index > 0
                && matches!(
                    words.get(index - 1),
                    Some(&"at") | Some(&"in") | Some(&"the") | Some(&"of") | Some(&"on")
                )
            {
                words.remove(index - 1);
                index -= 1;
            }
            continue;
        }
        index += 1;
    }
    (words.join(" "), hours)
}

/// Split a clock time ("17:00", "5:30") out of the words and return its hour
/// and minute. Only the form `format` writes is read: one or two hour digits,
/// a colon, two minute digits. A dangling "at" before it goes too.
fn split_clock_time(text: &str) -> (String, Option<(u32, u32)>) {
    let mut words: Vec<&str> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|w| !w.is_empty())
        .collect();
    let mut found = None;
    if let Some(index) = words.iter().position(|word| clock_time(word).is_some()) {
        found = clock_time(words[index]);
        words.remove(index);
        if index > 0 && words.get(index - 1) == Some(&"at") {
            words.remove(index - 1);
        }
    }
    (words.join(" "), found)
}

fn clock_time(word: &str) -> Option<(u32, u32)> {
    let (hour, minute) = word.split_once(':')?;
    let digits = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit());
    if hour.len() > 2 || minute.len() != 2 || !digits(hour) || !digits(minute) {
        return None;
    }
    let hour: u32 = hour.parse().ok()?;
    let minute: u32 = minute.parse().ok()?;
    (hour < 24 && minute < 60).then_some((hour, minute))
}

/// A number that may carry a leading minus: years before the epoch and day
/// counts before day zero, as `format` writes them.
fn parse_signed_number(word: &str) -> Option<i64> {
    match word.strip_prefix('-') {
        Some(rest) if !rest.is_empty() => parse_number(rest).map(|n| -n),
        _ => parse_number(word),
    }
}

/// Minutes to ticks at `per_day` ticks per day, rounding down or up. A span
/// finer than the tick widens to the ticks that could hold it.
fn ticks_from_minutes(minutes: Tick, per_day: Tick, round_up: bool) -> Tick {
    let minutes_per_tick = (1440 / per_day).max(1);
    if round_up {
        minutes
            .saturating_add(minutes_per_tick - 1)
            .div_euclid(minutes_per_tick)
    } else {
        minutes.div_euclid(minutes_per_tick)
    }
}

/// Numbers in story time are capped at nine digits: enough for any year or
/// day count a story needs, small enough that no tick arithmetic can overflow.
fn parse_number(word: &str) -> Option<i64> {
    let digits: String = word.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() > 9 {
        return None;
    }
    // Reject words that mix letters other than an ordinal suffix.
    let rest: String = word.chars().filter(|c| !c.is_ascii_digit()).collect();
    if !rest.is_empty() && !matches!(rest.as_str(), "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok()
}

fn number_word(word: &str) -> Option<i64> {
    Some(match word {
        "a" | "an" | "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        "twenty" => 20,
        "thirty" => 30,
        "forty" => 40,
        "fifty" => 50,
        "hundred" => 100,
        _ => return None,
    })
}

/// Returns (lo, hi, words consumed) for an amount phrase: "3", "three", "a few", "several".
fn parse_amount(words: &[&str]) -> Option<(Tick, Tick, usize)> {
    let first = *words.first()?;
    if first == "a" || first == "an" {
        match words.get(1).copied() {
            Some("few") => return Some((2, 6, 2)),
            Some("couple") => {
                let skip = usize::from(words.get(2).copied() == Some("of"));
                return Some((2, 3, 2 + skip));
            }
            _ => {}
        }
    }
    if let Some(number) = parse_number(first) {
        return Some((number, number, 1));
    }
    if let Some(number) = number_word(first) {
        return Some((number, number, 1));
    }
    match first {
        "few" => return Some((2, 6, 1)),
        "several" => return Some((3, 9, 1)),
        "some" => return Some((2, 12, 1)),
        "many" => return Some((6, 30, 1)),
        "half" => return Some((0, 1, 1)),
        "couple" => return Some((2, 3, 1)),
        _ => {}
    }
    None
}

/// The length of a unit word in minutes, the finest tick, with its granularity.
fn unit_minutes(word: &str) -> Option<(Tick, Granularity)> {
    const DAY: Tick = 1440;
    let word = word.trim_end_matches(',');
    let singular = word.strip_suffix('s').unwrap_or(word);
    Some(match singular {
        "minute" | "min" => (1, Granularity::Minute),
        "hour" | "hr" => (60, Granularity::Hour),
        "day" | "night" | "morning" | "evening" | "sunrise" | "sunset" | "dawn" => {
            (DAY, Granularity::Day)
        }
        "week" => (7 * DAY, Granularity::Week),
        "fortnight" => (14 * DAY, Granularity::Week),
        "month" | "moon" => (30 * DAY, Granularity::Month),
        "season" => (91 * DAY, Granularity::Season),
        "year" | "winter" | "summer" | "spring" | "autumn" | "fall" => {
            (365 * DAY, Granularity::Year)
        }
        "decade" => (3650 * DAY, Granularity::Year),
        "century" | "centurie" => (36500 * DAY, Granularity::Year),
        _ => return None,
    })
}

// ───────────────────────── day-count calendar ─────────────────────────

/// Returns inclusive day bounds.
fn parse_day_count(text: &str) -> Option<(i64, i64)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    match words.as_slice() {
        ["day", n] | ["d", n] | ["d.", n] => parse_signed_number(n).map(|n| (n, n)),
        [n, "day"] | [n] => parse_signed_number(n).map(|n| (n, n)),
        ["days", a, "-", b] | ["day", a, "-", b] | ["days", a, "to", b] | ["day", a, "to", b] => {
            Some((parse_signed_number(a)?, parse_signed_number(b)?))
        }
        ["week", n] => parse_number(n).map(|n| (n * 7, n * 7 + 6)),
        ["year", n] => parse_number(n).map(|n| (n * 365, n * 365 + 364)),
        _ => {
            // "day 14-16" written without spaces.
            if let Some(rest) = text.strip_prefix("day ") {
                if let Some((a, b)) = rest.split_once('-') {
                    return Some((parse_number(a.trim())?, parse_number(b.trim())?));
                }
            }
            None
        }
    }
}

// ───────────────────────── Gregorian ─────────────────────────

/// Days since 0000-03-01 for a proleptic Gregorian civil date (Hinnant's algorithm).
#[must_use]
pub fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(month) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe
}

#[must_use]
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 30,
    }
}

const MONTHS: &[&str] = &[
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

fn month_from_name(word: &str) -> Option<u32> {
    let word = word.trim_end_matches('.');
    MONTHS
        .iter()
        .position(|name| name.starts_with(word) && word.len() >= 3)
        .map(|index| index as u32 + 1)
}

fn season_bounds(word: &str, year: i64) -> Option<(i64, i64)> {
    let (from_month, to_month, to_year) = match word {
        "spring" => (3, 5, year),
        "summer" => (6, 8, year),
        "autumn" | "fall" => (9, 11, year),
        "winter" => (12, 2, year + 1),
        _ => return None,
    };
    let lo = days_from_civil(year, from_month, 1);
    let hi = days_from_civil(to_year, to_month, days_in_month(to_year, to_month));
    Some((lo, hi))
}

fn parse_gregorian(text: &str) -> Option<(i64, i64)> {
    let text = text.trim();
    // ISO forms: YYYY-MM-DD, YYYY-MM, YYYY, with a leading minus for years
    // before year 0 as `format` writes them. Years keep the nine-digit cap so
    // the day arithmetic cannot overflow.
    let (sign, body) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text),
    };
    let parts: Vec<&str> = body.split('-').collect();
    if parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
    {
        let iso_year = |year: &str| -> Option<i64> {
            if year.len() > 9 {
                return None;
            }
            year.parse::<i64>().ok().map(|y| sign * y)
        };
        match parts.as_slice() {
            [year, month, day] if year.len() >= 3 => {
                let y = iso_year(year)?;
                let m: u32 = month.parse().ok()?;
                let d: u32 = day.parse().ok()?;
                if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
                    return None;
                }
                let days = days_from_civil(y, m, d);
                return Some((days, days));
            }
            [year, month] if year.len() >= 3 => {
                let y = iso_year(year)?;
                let m: u32 = month.parse().ok()?;
                if !(1..=12).contains(&m) {
                    return None;
                }
                return Some((
                    days_from_civil(y, m, 1),
                    days_from_civil(y, m, days_in_month(y, m)),
                ));
            }
            [year] if year.len() >= 3 => {
                let y = iso_year(year)?;
                return Some((days_from_civil(y, 1, 1), days_from_civil(y, 12, 31)));
            }
            _ => {}
        }
    }
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty())
        .collect();
    match words.as_slice() {
        ["year", year] | ["in", year] | ["ad", year] | [year, "ad"] | [year, "ce"] => {
            let y = parse_number(year)?;
            Some((days_from_civil(y, 1, 1), days_from_civil(y, 12, 31)))
        }
        [year, "bc"] | [year, "bce"] => {
            let y = 1 - parse_number(year)?;
            Some((days_from_civil(y, 1, 1), days_from_civil(y, 12, 31)))
        }
        [season, "of", year] => season_bounds(season, parse_number(year)?),
        // "2 march 1890", "march 2 1890"
        [day, month, year] => {
            if let (Some(d), Some(m), Some(y)) = (
                parse_number(day),
                month_from_name(month),
                parse_number(year),
            ) {
                let d = u32::try_from(d).ok()?;
                if d == 0 || d > days_in_month(y, m) {
                    return None;
                }
                let days = days_from_civil(y, m, d);
                return Some((days, days));
            }
            if let (Some(m), Some(d), Some(y)) = (
                month_from_name(day),
                parse_number(month),
                parse_number(year),
            ) {
                let d = u32::try_from(d).ok()?;
                if d == 0 || d > days_in_month(y, m) {
                    return None;
                }
                let days = days_from_civil(y, m, d);
                return Some((days, days));
            }
            None
        }
        // "march 1890", "spring 1890"
        [month_or_season, year] => {
            let y = parse_number(year)?;
            if let Some(m) = month_from_name(month_or_season) {
                return Some((
                    days_from_civil(y, m, 1),
                    days_from_civil(y, m, days_in_month(y, m)),
                ));
            }
            season_bounds(month_or_season, y)
        }
        [single] => {
            let y = parse_number(single)?;
            if (100..=9999).contains(&y) {
                Some((days_from_civil(y, 1, 1), days_from_civil(y, 12, 31)))
            } else {
                None
            }
        }
        _ => None,
    }
}

// ───────────────────────── fictional calendars ─────────────────────────

fn year_days(calendar: &FictionalCalendar, year: i64) -> i64 {
    let base: i64 = calendar
        .months
        .iter()
        .map(|month| i64::from(month.days))
        .sum();
    base + leap_days(calendar, year)
}

fn leap_days(calendar: &FictionalCalendar, year: i64) -> i64 {
    match &calendar.leap {
        Some(rule) if rule.every > 0 && year.rem_euclid(i64::from(rule.every)) == 0 => {
            i64::from(rule.extra_days)
        }
        _ => 0,
    }
}

/// Days in a common year: the months without any leap addition.
fn base_year_days(calendar: &FictionalCalendar) -> i64 {
    calendar
        .months
        .iter()
        .map(|month| i64::from(month.days))
        .sum()
}

/// Leap years in the half-open range `[from, to)` with `from <= to`.
fn leap_years_between(calendar: &FictionalCalendar, from: i64, to: i64) -> i64 {
    match &calendar.leap {
        Some(rule) if rule.every > 0 && to > from => {
            let every = i64::from(rule.every);
            (to - 1).div_euclid(every) - (from - 1).div_euclid(every)
        }
        _ => 0,
    }
}

/// Days from the epoch to the first day of `year`, in closed form.
fn days_before_year(calendar: &FictionalCalendar, year: i64) -> i64 {
    let base = base_year_days(calendar);
    let extra = calendar
        .leap
        .as_ref()
        .map_or(0, |rule| i64::from(rule.extra_days));
    let (from, to, sign) = if year >= calendar.epoch_year {
        (calendar.epoch_year, year, 1)
    } else {
        (year, calendar.epoch_year, -1)
    };
    let years = to - from;
    let total = years
        .saturating_mul(base)
        .saturating_add(leap_years_between(calendar, from, to).saturating_mul(extra));
    total.saturating_mul(sign)
}

fn month_days(calendar: &FictionalCalendar, year: i64, month_index: usize) -> i64 {
    let base = calendar
        .months
        .get(month_index)
        .map(|m| i64::from(m.days))
        .unwrap_or(30);
    match &calendar.leap {
        Some(rule) if rule.month == month_index => base + leap_days(calendar, year),
        _ => base,
    }
}

fn fictional_days(calendar: &FictionalCalendar, year: i64, month_index: usize, day: i64) -> i64 {
    let mut days = days_before_year(calendar, year);
    for index in 0..month_index {
        days += month_days(calendar, year, index);
    }
    days + day - 1
}

fn fictional_from_days(calendar: &FictionalCalendar, days: i64) -> (i64, usize, i64) {
    let base = base_year_days(calendar);
    if calendar.months.is_empty() || base <= 0 {
        return (calendar.epoch_year, 0, days.saturating_add(1));
    }
    // Estimate the year from the average year length, then settle it with a
    // few bounded steps. The estimate is exact over whole leap cycles, so the
    // correction never needs more than a handful of iterations.
    let average = match &calendar.leap {
        Some(rule) if rule.every > 0 => {
            base as f64 + f64::from(rule.extra_days) / f64::from(rule.every)
        }
        _ => base as f64,
    };
    let mut year = calendar
        .epoch_year
        .saturating_add((days as f64 / average).floor() as i64);
    for _ in 0..64 {
        if days < days_before_year(calendar, year) {
            year -= 1;
        } else if days >= days_before_year(calendar, year.saturating_add(1)) {
            year += 1;
        } else {
            break;
        }
    }
    let mut remaining = days - days_before_year(calendar, year);
    let mut month_index = 0;
    while month_index + 1 < calendar.months.len() {
        let length = month_days(calendar, year, month_index);
        if remaining < length {
            break;
        }
        remaining -= length;
        month_index += 1;
    }
    (year, month_index, remaining + 1)
}

fn parse_fictional(calendar: &FictionalCalendar, text: &str) -> Option<(i64, i64)> {
    // ISO-like "412-03-01": year-month-day numerically in the fictional calendar.
    let (sign, body) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text),
    };
    let parts: Vec<&str> = body.split('-').collect();
    if let [year, month, day] = parts.as_slice() {
        if parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
        {
            let y = sign * parse_number(year)?;
            let m = parse_number(month)? as usize;
            let d = parse_number(day)?;
            if m == 0 || m > calendar.months.len() {
                return None;
            }
            let days = fictional_days(calendar, y, m - 1, d);
            return Some((days, days));
        }
    }
    let mut words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty() && *w != "of" && *w != "the")
        .collect();
    // Strip an era suffix.
    if let Some(last) = words.last() {
        if calendar.eras.iter().any(|era| era.to_lowercase() == *last) {
            words.pop();
        }
    }
    // Names fold per scalar as `normalise` folds the label. An exact name wins
    // over a prefix, so "Sun" is not read as "Sunder" when both are months.
    let folded = |name: &str| -> String { name.chars().flat_map(char::to_lowercase).collect() };
    let month_index = |word: &str| -> Option<usize> {
        calendar
            .months
            .iter()
            .position(|month| folded(&month.name) == word)
            .or_else(|| {
                if word.len() < 3 {
                    return None;
                }
                calendar
                    .months
                    .iter()
                    .position(|month| folded(&month.name).starts_with(word))
            })
    };
    let season_index = |word: &str| -> Option<usize> {
        calendar
            .seasons
            .iter()
            .position(|season| season.to_lowercase() == word)
    };
    match words.as_slice() {
        ["year", year] | [year, "year"] => {
            let y = parse_signed_number(year)?;
            let lo = days_before_year(calendar, y);
            Some((lo, lo + year_days(calendar, y) - 1))
        }
        ["day", n] => {
            let n = parse_signed_number(n)?;
            Some((n, n))
        }
        [day, month, year] => {
            if let (Some(d), Some(m), Some(y)) = (
                parse_number(day),
                month_index(month),
                parse_signed_number(year),
            ) {
                let days = fictional_days(calendar, y, m, d);
                return Some((days, days));
            }
            if let (Some(m), Some(d), Some(y)) = (
                month_index(day),
                parse_number(month),
                parse_signed_number(year),
            ) {
                let days = fictional_days(calendar, y, m, d);
                return Some((days, days));
            }
            None
        }
        [month_or_season, year] => {
            let y = parse_signed_number(year)?;
            if let Some(m) = month_index(month_or_season) {
                let lo = fictional_days(calendar, y, m, 1);
                return Some((lo, lo + month_days(calendar, y, m) - 1));
            }
            if let Some(index) = season_index(month_or_season) {
                let count = calendar.seasons.len() as i64;
                let length = year_days(calendar, y);
                let lo = days_before_year(calendar, y) + length * index as i64 / count;
                let hi = days_before_year(calendar, y) + length * (index as i64 + 1) / count - 1;
                return Some((lo, hi));
            }
            None
        }
        [single] => {
            let y = parse_signed_number(single)?;
            let lo = days_before_year(calendar, y);
            Some((lo, lo + year_days(calendar, y) - 1))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_month_calendar() -> FictionalCalendar {
        FictionalCalendar {
            months: vec![
                Month {
                    name: "First".into(),
                    days: 30,
                },
                Month {
                    name: "Second".into(),
                    days: 30,
                },
            ],
            leap: Some(LeapRule {
                every: 4,
                extra_days: 1,
                month: 1,
            }),
            epoch_year: 1,
            eras: Vec::new(),
            seasons: Vec::new(),
        }
    }

    #[test]
    fn fictional_years_are_closed_form() {
        let calendar = two_month_calendar();
        // 60-day years, one extra day every fourth year.
        assert_eq!(days_before_year(&calendar, 1), 0);
        assert_eq!(days_before_year(&calendar, 2), 60);
        assert_eq!(days_before_year(&calendar, 5), 60 * 4 + 1);
        assert_eq!(days_before_year(&calendar, 9), 60 * 8 + 2);
        assert_eq!(days_before_year(&calendar, -3), -(60 * 4 + 1));
        for year in [-7, -1, 1, 2, 4, 5, 8, 9, 400, 999_999_999] {
            for (month, day) in [(0, 1), (1, 30)] {
                let days = fictional_days(&calendar, year, month, day);
                assert_eq!(fictional_from_days(&calendar, days), (year, month, day));
            }
        }
        // A twelve-digit year is rejected rather than walked year by year.
        let clock = Clock::new(TimeUnit::Day, Calendar::Fictional(calendar.clone()));
        assert!(clock.parse("year 999999999999").is_none());
        let started = std::time::Instant::now();
        let _ = clock.format(1_000_000_000_000);
        assert!(started.elapsed().as_millis() < 500);
        // Zero-day months are refused by validation and tolerated by format.
        let mut broken = calendar;
        broken.months[0].days = 0;
        assert!(broken.validate().is_err());
        broken.months[1].days = 0;
        let clock = Clock::new(TimeUnit::Day, Calendar::Fictional(broken));
        let _ = clock.format(500);
    }

    fn day_clock() -> Clock {
        Clock::new(TimeUnit::Day, Calendar::DayCount)
    }

    #[test]
    fn parses_day_counts_in_any_script() {
        let clock = day_clock();
        assert_eq!(
            clock.parse("Day 14"),
            Some(Parsed::Absolute(TimeBounds::at(14)))
        );
        assert_eq!(
            clock.parse("day ১৪"),
            Some(Parsed::Absolute(TimeBounds::at(14)))
        );
        assert_eq!(
            clock.parse("Day 3, dusk"),
            Some(Parsed::Absolute(TimeBounds::at(3)))
        );
        assert_eq!(
            clock.parse("about day 10"),
            Some(Parsed::Absolute(TimeBounds::between(7, 13)))
        );
        assert_eq!(
            clock.parse("days 4 to 9"),
            Some(Parsed::Absolute(TimeBounds::between(4, 9)))
        );
    }

    #[test]
    fn hours_refine_time_of_day() {
        let clock = Clock::new(TimeUnit::Hour, Calendar::DayCount);
        assert_eq!(
            clock.parse("day 3, dusk"),
            Some(Parsed::Absolute(TimeBounds::between(
                3 * 24 + 17,
                3 * 24 + 19
            )))
        );
        assert_eq!(clock.format(3 * 24 + 17), "Day 3 17:00");
    }

    #[test]
    fn parses_relative_labels() {
        let clock = day_clock();
        assert_eq!(
            clock.parse("three days after the siege"),
            Some(Parsed::Relative {
                relation: Relation::After,
                lo: 3,
                hi: 3,
                anchor: Some("siege".into()),
            })
        );
        assert_eq!(
            clock.parse("a few days later"),
            Some(Parsed::Relative {
                relation: Relation::After,
                lo: 2,
                hi: 6,
                anchor: None,
            })
        );
        assert_eq!(
            clock.parse("the next morning"),
            Some(Parsed::Relative {
                relation: Relation::After,
                lo: 1,
                hi: 1,
                anchor: None,
            })
        );
        assert_eq!(
            clock.parse("two years before the treaty"),
            Some(Parsed::Relative {
                relation: Relation::Before,
                lo: 730,
                hi: 730,
                anchor: Some("treaty".into()),
            })
        );
    }

    #[test]
    fn gregorian_round_trips() {
        let clock = Clock::new(TimeUnit::Day, Calendar::Gregorian);
        let days = days_from_civil(1890, 3, 2);
        assert_eq!(civil_from_days(days), (1890, 3, 2));
        assert_eq!(
            clock.parse("1890-03-02"),
            Some(Parsed::Absolute(TimeBounds::at(days)))
        );
        assert_eq!(
            clock.parse("2 March 1890"),
            Some(Parsed::Absolute(TimeBounds::at(days)))
        );
        assert_eq!(clock.format(days), "1890-03-02");
        let Some(Parsed::Absolute(winter)) = clock.parse("winter of 1890") else {
            panic!("expected a season");
        };
        assert_eq!(winter.lo, Some(days_from_civil(1890, 12, 1)));
        assert_eq!(winter.hi, Some(days_from_civil(1891, 2, 28)));
        let Some(Parsed::Absolute(year)) = clock.parse("around 1890") else {
            panic!("expected a fuzzy year");
        };
        assert!(year.lo.unwrap() < days_from_civil(1890, 1, 1));
    }

    #[test]
    fn fictional_calendar_counts_months_and_leaps() {
        let calendar = FictionalCalendar {
            months: vec![
                Month {
                    name: "Thawmonth".into(),
                    days: 30,
                },
                Month {
                    name: "Seedmonth".into(),
                    days: 30,
                },
                Month {
                    name: "Harvest".into(),
                    days: 31,
                },
            ],
            leap: Some(LeapRule {
                every: 4,
                extra_days: 1,
                month: 2,
            }),
            epoch_year: 400,
            eras: vec!["AE".into()],
            seasons: vec!["thaw".into(), "high".into(), "fall".into()],
        };
        let clock = Clock::new(TimeUnit::Day, Calendar::Fictional(calendar));
        // Year 400 has 92 days (leap), 401 has 91.
        assert_eq!(
            clock.parse("1 Thawmonth 401 AE"),
            Some(Parsed::Absolute(TimeBounds::at(92)))
        );
        assert_eq!(
            clock.parse("Harvest 400"),
            Some(Parsed::Absolute(TimeBounds::between(60, 91)))
        );
        assert_eq!(clock.format(92), "1 Thawmonth 401");
        assert_eq!(clock.parse("412-02-05"), clock.parse("5 Seedmonth 412"));
    }
}
