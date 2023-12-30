use self::ParseDurationError::*;
use crate::error::Error;
use std::{error, fmt, time::Duration};

/// An error resulting from parsing a duration from a string.
#[derive(Debug)]
pub enum ParseDurationError {
    /// Invalid number.
    InvalidNumber(Error),
    /// Invalid unit.
    InvalidUnit(String),
    /// Invalid format.
    InvalidFormat,
}

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidNumber(err) => write!(f, "invalid number: {err}"),
            InvalidUnit(err) => write!(f, "invalid unit: {err}"),
            InvalidFormat => write!(f, "invalid format"),
        }
    }
}

impl error::Error for ParseDurationError {}

/// Parses a duration from a string.
///
/// The input string is specified as an integer followed immediately by one of the following units:
///
/// - `ms` - milliseconds
/// - `s` - seconds
/// - `m` - minutes
/// - `h` - hours
/// - `d` - days
/// - `w` - weeks
///
/// Units must be ordered from the longest to the shortest, and
/// a given unit must only appear once in a time duration.
pub fn parse_duration(mut input: &str) -> Result<Duration, ParseDurationError> {
    const UNIT_IN_MILLIS: [u64; 7] = [0, 604_800_000, 86_400_000, 3_600_000, 60_000, 1_000, 1];
    let mut nonterminated = true;
    let mut last_unit_order = 0;
    let mut milliseconds = 0;
    while nonterminated {
        let Some(index) = input.find(|ch: char| ch.is_alphabetic()) else {
            break;
        };
        let (number, remainder) = input.split_at(index);
        let number = number
            .parse::<u64>()
            .map_err(|err| InvalidNumber(err.into()))?;
        let unit = if let Some(index) = remainder.find(|ch: char| ch.is_ascii_digit()) {
            let (unit, remainder) = remainder.split_at(index);
            input = remainder;
            unit
        } else {
            nonterminated = false;
            remainder
        };
        let unit_order = if unit == "ms" {
            UNIT_IN_MILLIS.len() - 1
        } else {
            "wdhms"
                .find(unit)
                .map(|index| index + 1)
                .unwrap_or_default()
        };
        if unit_order > last_unit_order {
            milliseconds += number * UNIT_IN_MILLIS[unit_order];
            last_unit_order = unit_order;
        } else if nonterminated {
            return Err(InvalidUnit(format!(
                "unit `{unit}` in `{remainder}` is not allowed"
            )));
        }
    }
    Ok(Duration::from_millis(milliseconds))
}

#[cfg(test)]
mod tests {
    use super::parse_duration;
    use std::time::Duration;

    #[test]
    fn it_parses_duration() {
        assert_eq!(parse_duration("1h30m").unwrap(), Duration::from_secs(5400));
        assert_eq!(
            parse_duration("20s500ms").unwrap(),
            Duration::from_millis(20500),
        );
        assert!(parse_duration("6.5h").is_err());
    }
}
