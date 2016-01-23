use std::borrow::Cow;
use std::env;
use std::path::Path;

use chrono::Duration;

#[inline]
pub fn str_to_path(s: &str) -> Cow<Path> {
    if s.starts_with("~/") {
        if let Some(mut home_dir) = env::home_dir() {
            home_dir.push(&s[2..]);
            return home_dir.into();
        } 
    }
    Path::new(s).into()
}

#[inline]
pub fn str_to_path_0(s: &String) -> Cow<Path> { str_to_path(s) }

pub fn parse_duration(s: &str) -> Option<Duration> {
    let mut digits = String::new();
    let mut unit = String::new();
    let mut reading_digits = true;
    for c in s.chars().filter(|c| !c.is_whitespace()) {
        if reading_digits {
            if c.is_digit(10) {
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

