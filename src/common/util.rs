use std::borrow::Cow;
use std::path::{Path, PathBuf};

use chrono::{Duration, DateTime, Utc};
use shellexpand;

#[inline]
pub fn str_to_path(s: &str) -> Cow<Path> {
    match shellexpand::tilde(s) {
        Cow::Borrowed(s) => Path::new(s).into(),
        Cow::Owned(s) => PathBuf::from(s).into(),
    }
}

#[inline]
pub fn string_to_path(s: &String) -> Cow<Path> { str_to_path(s) }

const BASE_TEN: u32 = 10;

pub fn parse_duration(s: &str) -> Option<Duration> {
    let mut digits = String::new();
    let mut unit = String::new();
    let mut reading_digits = true;
    for c in s.chars().filter(|c| !c.is_whitespace()) {
        if reading_digits {
            if c.is_digit(BASE_TEN) {
                digits.push(c);
            } else {
                reading_digits = false;
                unit.push(c)
            }
        } else {
            unit.push(c);
        }
    }

    if digits.is_empty() { return None; }

    let mk_duration: fn(i64) -> Duration = match &*unit {
        "ns" | "nanos" | "nano" | "nanoseconds" | "nanosecond" => Duration::nanoseconds,
        "us" | "micros" | "micro" | "microseconds" | "microsecond" => Duration::microseconds,
        "ms" | "millis" | "milli" | "milliseconds" | "millisecond" => Duration::milliseconds,
        "s" | "secs" | "sec" | "seconds" | "second" => Duration::seconds,
        "m" | "mins" | "min" | "minutes" | "minute" => Duration::minutes,
        "h" | "hours" | "hour" => Duration::hours,
        "d" | "days" | "day" => Duration::days,
        _ => return None
    };

    let n = digits.parse().unwrap();  // always correct
    Some(mk_duration(n))
}

// returns an instant most likely located in the past
pub fn past_timestamp() -> DateTime<Utc> {
    DateTime::parse_from_str("0+0000", "%s%z").unwrap().with_timezone(&Utc)
}
