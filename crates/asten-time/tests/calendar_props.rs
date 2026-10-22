//! Property tests for calendars and labels: `parse(format(tick))` round trips
//! in every calendar and unit, civil-date arithmetic inverts, and relative
//! expressions scale with the clock unit.

use asten_ir::{Tick, TimeBounds};
use asten_time::calendar::{
    Calendar, Clock, FictionalCalendar, LeapRule, Month, Parsed, Relation, TimeUnit,
    civil_from_days, days_from_civil,
};
use proptest::prelude::*;

const CASES: u32 = 256;

// ───────────────────────── generators ─────────────────────────

fn unit() -> impl Strategy<Value = TimeUnit> {
    prop_oneof![
        Just(TimeUnit::Day),
        Just(TimeUnit::Hour),
        Just(TimeUnit::Minute)
    ]
}

/// Ticks from narrow, medium and wide ranges, negative as often as positive.
fn tick() -> impl Strategy<Value = Tick> {
    prop_oneof![
        2 => -400..400i64,
        2 => -2_000_000..2_000_000i64,
        1 => -100_000_000..100_000_000i64,
    ]
}

/// Words the label grammar reserves; a month with one of these names cannot
/// be read back and the generator avoids them (a documented limit).
const RESERVED: &[&str] = &[
    "of",
    "the",
    "day",
    "year",
    "midnight",
    "dawn",
    "sunrise",
    "morning",
    "noon",
    "midday",
    "afternoon",
    "dusk",
    "sunset",
    "evening",
    "night",
];

fn month_name() -> impl Strategy<Value = String> {
    ("[a-z]{1,8}", any::<bool>())
        .prop_filter("reserved", |(name, _)| !RESERVED.contains(&name.as_str()))
        .prop_map(|(name, capitalise)| {
            if capitalise {
                let mut chars = name.chars();
                let first = chars.next().unwrap().to_ascii_uppercase();
                std::iter::once(first).chain(chars).collect()
            } else {
                name
            }
        })
}

fn fictional() -> impl Strategy<Value = FictionalCalendar> {
    (1..=13usize)
        .prop_flat_map(|count| {
            (
                prop::collection::hash_set(month_name(), count),
                prop::collection::vec(1..=60u32, count),
                prop::option::of((1..=12u32, 1..=5u32, 0..count)),
                -1000..=3000i64,
                prop::sample::subsequence(vec!["AE", "BE", "NE"], 0..=2),
            )
        })
        .prop_filter("distinct lowercase names", |(names, ..)| {
            let lower: std::collections::HashSet<String> =
                names.iter().map(|name| name.to_lowercase()).collect();
            lower.len() == names.len()
        })
        .prop_map(|(names, days, leap, epoch_year, eras)| {
            let mut names: Vec<String> = names.into_iter().collect();
            names.sort();
            let months = names
                .into_iter()
                .zip(days)
                .map(|(name, days)| Month { name, days })
                .collect();
            FictionalCalendar {
                months,
                leap: leap.map(|(every, extra_days, month)| LeapRule {
                    every,
                    extra_days,
                    month,
                }),
                epoch_year,
                eras: eras.into_iter().map(str::to_owned).collect(),
                seasons: Vec::new(),
            }
        })
}

fn calendar() -> impl Strategy<Value = Calendar> {
    prop_oneof![
        1 => Just(Calendar::DayCount),
        2 => Just(Calendar::Gregorian),
        3 => fictional().prop_map(Calendar::Fictional),
    ]
}

// ───────────────────────── absolute labels ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// Every tick formats to a label that parses back to exactly that tick, in
    /// every calendar and at every unit.
    #[test]
    fn format_then_parse_is_identity(calendar in calendar(), unit in unit(), tick in tick()) {
        if let Calendar::Fictional(fictional) = &calendar {
            prop_assume!(fictional.validate().is_ok());
        }
        let clock = Clock::new(unit, calendar);
        let label = clock.format(tick);
        prop_assert_eq!(
            clock.parse(&label),
            Some(Parsed::Absolute(TimeBounds::at(tick))),
            "label {:?} for tick {}", label, tick
        );
        // Day labels name whole days: the parse of the bare date is the day's span.
        if unit != TimeUnit::Day {
            let day_clock = Clock::new(TimeUnit::Day, clock.calendar.clone());
            let day = tick.div_euclid(unit.ticks_per_day());
            prop_assert_eq!(
                day_clock.parse(&day_clock.format(day)),
                Some(Parsed::Absolute(TimeBounds::at(day)))
            );
        }
    }

    /// Fictional dates stay inside their month and the formatted day is 1-based.
    #[test]
    fn fictional_days_stay_inside_their_month(fictional in fictional(), tick in tick()) {
        prop_assume!(fictional.validate().is_ok());
        let clock = Clock::new(TimeUnit::Day, Calendar::Fictional(fictional.clone()));
        let label = clock.format(tick);
        let mut words = label.split(' ');
        let dom: i64 = words.next().unwrap().parse().unwrap();
        let year: i64 = words.next_back().unwrap().parse().unwrap();
        let month: String = words.collect::<Vec<_>>().join(" ");
        let index = fictional.months.iter().position(|m| m.name == month).expect("a month name");
        let mut length = i64::from(fictional.months[index].days);
        if let Some(rule) = &fictional.leap {
            if rule.month == index && year.rem_euclid(i64::from(rule.every)) == 0 {
                length += i64::from(rule.extra_days);
            }
        }
        prop_assert!(dom >= 1 && dom <= length, "{label}: day {dom} of a {length}-day month");
        // Consecutive ticks either advance the day by one or start a new month/year.
        let next = clock.format(tick + 1);
        let next_dom: i64 = next.split(' ').next().unwrap().parse().unwrap();
        prop_assert!(next_dom == dom + 1 || next_dom == 1, "{label} -> {next}");
    }

    #[test]
    fn civil_dates_round_trip(year in -1_000_000..1_000_000i64, month in 1..=12u32, day in 1..=31u32) {
        let last = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            _ if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
            _ => 28,
        };
        let day = day.min(last);
        let days = days_from_civil(year, month, day);
        prop_assert_eq!(civil_from_days(days), (year, month, day));
        // The next day is the next count, and 1 March 0000 is day 0.
        prop_assert_eq!(days_from_civil(0, 3, 1), 0);
        let (y2, m2, d2) = civil_from_days(days + 1);
        prop_assert!(days_from_civil(y2, m2, d2) == days + 1);
    }

    #[test]
    fn day_counts_round_trip(days in -1_000_000_000..1_000_000_000i64) {
        let (year, month, day) = civil_from_days(days);
        prop_assert!((1..=12).contains(&month) && (1..=31).contains(&day));
        prop_assert_eq!(days_from_civil(year, month, day), days);
    }
}

// ───────────────────────── relative labels ─────────────────────────

/// An amount phrase with its tick-free bounds.
fn amount() -> impl Strategy<Value = (String, i64, i64)> {
    prop_oneof![
        3 => (1..=999_999_999i64).prop_map(|n| (n.to_string(), n, n)),
        1 => prop::sample::select(vec![
            ("one", 1), ("two", 2), ("three", 3), ("four", 4), ("five", 5), ("six", 6),
            ("seven", 7), ("eight", 8), ("nine", 9), ("ten", 10), ("eleven", 11),
            ("twelve", 12), ("twenty", 20), ("thirty", 30), ("forty", 40), ("fifty", 50),
            ("hundred", 100), ("a", 1), ("an", 1),
        ]).prop_map(|(word, n)| (word.to_owned(), n, n)),
        1 => prop::sample::select(vec![
            ("a few", 2, 6), ("few", 2, 6), ("several", 3, 9), ("some", 2, 12),
            ("many", 6, 30), ("half", 0, 1), ("a couple of", 2, 3), ("a couple", 2, 3),
            ("couple", 2, 3),
        ]).prop_map(|(word, lo, hi)| (word.to_owned(), lo, hi)),
    ]
}

/// A unit word with its length in minutes, the finest tick.
fn time_unit_word() -> impl Strategy<Value = (String, i64)> {
    prop::sample::select(vec![
        ("minute", 1),
        ("min", 1),
        ("hour", 60),
        ("hr", 60),
        ("day", 1440),
        ("night", 1440),
        ("morning", 1440),
        ("evening", 1440),
        ("week", 7 * 1440),
        ("fortnight", 14 * 1440),
        ("month", 30 * 1440),
        ("moon", 30 * 1440),
        ("season", 91 * 1440),
        ("year", 365 * 1440),
        ("winter", 365 * 1440),
        ("summer", 365 * 1440),
        ("decade", 3650 * 1440),
        ("century", 36500 * 1440),
    ])
    .prop_flat_map(|(word, minutes)| {
        (any::<bool>()).prop_map(move |plural| {
            let form = if plural {
                match word {
                    "century" => "centuries".to_owned(),
                    other => format!("{other}s"),
                }
            } else {
                word.to_owned()
            };
            (form, minutes)
        })
    })
}

fn direction() -> impl Strategy<Value = (&'static str, Relation)> {
    prop::sample::select(vec![
        ("after", Relation::After),
        ("since", Relation::After),
        ("following", Relation::After),
        ("past", Relation::After),
        ("from", Relation::After),
        ("into", Relation::After),
        ("later", Relation::After),
        ("afterwards", Relation::After),
        ("hence", Relation::After),
        ("before", Relation::Before),
        ("earlier", Relation::Before),
        ("prior", Relation::Before),
        ("ago", Relation::Before),
        ("previously", Relation::Before),
        ("back", Relation::Before),
    ])
}

fn anchor() -> impl Strategy<Value = (String, Option<String>)> {
    prop_oneof![
        Just((String::new(), None)),
        Just((" the siege".to_owned(), Some("siege".to_owned()))),
        Just((" siege".to_owned(), Some("siege".to_owned()))),
        Just((
            " to the first feast".to_owned(),
            Some("first feast".to_owned())
        )),
        Just((
            " the treaty of veld".to_owned(),
            Some("treaty of veld".to_owned())
        )),
    ]
}

fn div_floor(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}

fn div_ceil(a: i64, b: i64) -> i64 {
    -((-a).div_euclid(b))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// "<amount> <unit> <direction> [anchor]" parses on every clock to the
    /// same relation and anchor, and the amounts obey one scaling law: exact
    /// in minutes, rounded outwards (floor of the low bound, ceiling of the
    /// high bound) at coarser units. Whole-day units are therefore exact on
    /// every clock; sub-tick units widen to the enclosing ticks.
    #[test]
    fn relative_amounts_scale_with_the_unit(
        (amount, lo, hi) in amount(),
        (unit_word, minutes) in time_unit_word(),
        (direction, relation) in direction(),
        (anchor_text, anchor) in anchor(),
    ) {
        let label = format!("{amount} {unit_word} {direction}{anchor_text}");
        let lo_minutes = lo.checked_mul(minutes).expect("fits");
        let hi_minutes = hi.checked_mul(minutes).expect("fits");
        for (unit, per_day) in [(TimeUnit::Minute, 1440), (TimeUnit::Hour, 24), (TimeUnit::Day, 1)] {
            let clock = Clock::new(unit, Calendar::DayCount);
            let parsed = clock.parse(&label);
            let divisor = 1440 / per_day;
            let expected = Parsed::Relative {
                relation,
                lo: div_floor(lo_minutes, divisor),
                hi: div_ceil(hi_minutes, divisor),
                anchor: anchor.clone(),
            };
            prop_assert_eq!(parsed, Some(expected), "{:?} on a {:?} clock", label, unit);
        }
    }

    /// The same label on the three clocks: the day-clock bounds enclose the
    /// hour-clock bounds scaled down, which enclose the minute-clock bounds.
    #[test]
    fn coarser_clocks_enclose_finer_ones(
        (amount, _, _) in amount(),
        (unit_word, _) in time_unit_word(),
        (direction, _) in direction(),
    ) {
        let label = format!("{amount} {unit_word} {direction}");
        let bounds = |unit: TimeUnit| match Clock::new(unit, Calendar::DayCount).parse(&label) {
            Some(Parsed::Relative { lo, hi, .. }) => (lo, hi),
            other => panic!("{label:?}: {other:?}"),
        };
        let (min_lo, min_hi) = bounds(TimeUnit::Minute);
        let (hour_lo, hour_hi) = bounds(TimeUnit::Hour);
        let (day_lo, day_hi) = bounds(TimeUnit::Day);
        prop_assert!(hour_lo * 60 <= min_lo && min_hi <= hour_hi * 60);
        prop_assert!(day_lo * 24 <= hour_lo && hour_hi <= day_hi * 24);
        prop_assert!(min_lo <= min_hi && hour_lo <= hour_hi && day_lo <= day_hi);
    }
}

/// Amounts are capped at nine digits, so the arithmetic never overflows; a
/// longer number is not a time at all rather than a saturated one.
#[test]
fn oversized_amounts_are_rejected_not_wrapped() {
    for unit in [TimeUnit::Minute, TimeUnit::Hour, TimeUnit::Day] {
        let clock = Clock::new(unit, Calendar::DayCount);
        assert_eq!(clock.parse("9999999999 centuries later"), None);
        let Some(Parsed::Relative { lo, hi, .. }) = clock.parse("999999999 centuries later") else {
            panic!("nine digits parse");
        };
        assert!(lo > 0 && lo == hi, "{unit:?}: {lo}..{hi}");
        assert_eq!(
            lo,
            999_999_999 * 36_500 * unit.ticks_per_day(),
            "{unit:?} scales centuries exactly"
        );
    }
}
