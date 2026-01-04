use std::{
    num::ParseIntError,
    time::{Duration, SystemTime},
};

use regex::Regex;
use time::{OffsetDateTime, error::ComponentRange, format_description::well_known};

pub trait ParseSystemTime {
    fn parse_system_time(&self) -> Result<SystemTime, TimeFormatError>;
}

#[derive(Debug, PartialEq)]
pub enum TimeFormat<'a> {
    UnixTimestamp(&'a str),
    Rfc3339(&'a str),
    DateOnly(&'a str),
    InvalidFormat,
}

#[derive(Debug, PartialEq)]
pub enum TimeFormatError {
    InvalidFormat,
}

impl From<ParseIntError> for TimeFormatError {
    fn from(_value: ParseIntError) -> Self {
        TimeFormatError::InvalidFormat
    }
}

impl From<ComponentRange> for TimeFormatError {
    fn from(_value: ComponentRange) -> Self {
        TimeFormatError::InvalidFormat
    }
}

impl<'a> TimeFormat<'a> {
    pub fn new(s: &'a str) -> Self {
        let unix_timestamp_regex = Regex::new(r"^\d{10,13}$").expect("Invalid Timestamp Regex");
        let rfc3339_regex =
            Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$")
                .expect("Invalid RFC 3339 Regex");
        let date_only_regex = Regex::new(r"^\d+-\d+-\d+$").expect("Invalid Date Only Regex");

        if unix_timestamp_regex.is_match(s) {
            Self::UnixTimestamp(s)
        } else if rfc3339_regex.is_match(s) {
            Self::Rfc3339(s)
        } else if date_only_regex.is_match(s) {
            Self::DateOnly(s)
        } else {
            Self::InvalidFormat
        }
    }
}

impl<'a> ParseSystemTime for TimeFormat<'a> {
    fn parse_system_time(&self) -> Result<SystemTime, TimeFormatError> {
        fn parse_offset_date_time(dt: OffsetDateTime) -> Result<SystemTime, TimeFormatError> {
            let unix = dt.unix_timestamp();
            let nanos = dt.nanosecond();

            if unix < 0 {
                Err(TimeFormatError::InvalidFormat)?;
            }

            Ok(SystemTime::UNIX_EPOCH + Duration::new(unix as u64, nanos))
        }

        fn parse_unix_timestamp(s: &str) -> Result<SystemTime, TimeFormatError> {
            let value: i128 = s.parse()?;

            let (secs, nanos) = match s.len() {
                0..=10 => (value, 0),
                11..=13 => (value / 1_000, (value % 1_000) * 1_000_000),
                _ => Err(TimeFormatError::InvalidFormat)?,
            };

            if secs < 0 {
                Err(TimeFormatError::InvalidFormat)?;
            }

            Ok(SystemTime::UNIX_EPOCH + Duration::new(secs as u64, nanos as u32))
        }

        fn parse_rfc3339(s: &str) -> Result<SystemTime, TimeFormatError> {
            let dt = OffsetDateTime::parse(s, &well_known::Rfc3339)
                .map_err(|_| TimeFormatError::InvalidFormat)?;

            parse_offset_date_time(dt)
        }

        fn parse_date_only(s: &str) -> Result<SystemTime, TimeFormatError> {
            let mut parts = s.split('-');

            let year: i32 = parts
                .next()
                .ok_or(TimeFormatError::InvalidFormat)?
                .parse()?;
            let month: u8 = parts
                .next()
                .ok_or(TimeFormatError::InvalidFormat)?
                .parse()?;
            let day: u8 = parts
                .next()
                .ok_or(TimeFormatError::InvalidFormat)?
                .parse()?;

            parts
                .next()
                .map_or(Ok(()), |_| Err(TimeFormatError::InvalidFormat))?;

            let month = time::Month::try_from(month)?;
            let date = time::Date::from_calendar_date(year, month, day)?;

            let rfc3339 = date
                .with_hms(0, 0, 0)
                .map_err(|_| TimeFormatError::InvalidFormat)?
                .assume_utc();

            parse_offset_date_time(rfc3339)
        }

        match self {
            TimeFormat::UnixTimestamp(s) => parse_unix_timestamp(s),
            TimeFormat::Rfc3339(s) => parse_rfc3339(s),
            TimeFormat::DateOnly(s) => parse_date_only(s),
            TimeFormat::InvalidFormat => Err(TimeFormatError::InvalidFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("2026-01-01", TimeFormat::DateOnly("2026-01-01"))]
    #[case(
        "2026-01-01T12:00:00+08:00",
        TimeFormat::Rfc3339("2026-01-01T12:00:00+08:00")
    )]
    #[case(
        "2026-01-01T12:00:00.123+08:00",
        TimeFormat::Rfc3339("2026-01-01T12:00:00.123+08:00")
    )]
    #[case(
        "2026-01-01T12:00:00-03:00",
        TimeFormat::Rfc3339("2026-01-01T12:00:00-03:00")
    )]
    #[case(
        "2026-01-01T12:00:00.123-03:00",
        TimeFormat::Rfc3339("2026-01-01T12:00:00.123-03:00")
    )]
    #[case("2026-01-01T12:00:00Z", TimeFormat::Rfc3339("2026-01-01T12:00:00Z"))]
    #[case(
        "2026-01-01T12:00:00.123Z",
        TimeFormat::Rfc3339("2026-01-01T12:00:00.123Z")
    )]
    #[case("1767296965", TimeFormat::UnixTimestamp("1767296965"))]
    #[case("17672969655", TimeFormat::UnixTimestamp("17672969655"))]
    #[case("176729696559", TimeFormat::UnixTimestamp("176729696559"))]
    #[case("1767296965592", TimeFormat::UnixTimestamp("1767296965592"))]
    #[case("2026-01-01-01", TimeFormat::InvalidFormat)]
    #[case("2026-01", TimeFormat::InvalidFormat)]
    #[case("2026", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T12:00.123+08:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T12.123+08:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T12:00+08:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T12+08:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T-03:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01T", TimeFormat::InvalidFormat)]
    #[case("2026-01-01X12:00:00.123-03:00", TimeFormat::InvalidFormat)]
    #[case("2026-01-01X12:00:00-03:00", TimeFormat::InvalidFormat)]
    #[case("176729696", TimeFormat::InvalidFormat)]
    #[case("17672969655922", TimeFormat::InvalidFormat)]
    fn should_create_time_format_from_string(
        #[case] s: &str,
        #[case] expected_time_format: TimeFormat,
    ) {
        // Arrange

        // Act
        let time_format = TimeFormat::new(s);

        // Assert
        assert_eq!(time_format, expected_time_format);
    }

    #[rstest]
    #[case(TimeFormat::DateOnly("2026-01-01"), 1767225600000)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56+08:00"), 1767242096000)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56.789+08:00"), 1767242096789)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56-03:00"), 1767281696000)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56.789-03:00"), 1767281696789)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56Z"), 1767270896000)]
    #[case(TimeFormat::Rfc3339("2026-01-01T12:34:56.789Z"), 1767270896789)]
    #[case(TimeFormat::UnixTimestamp("1767296965"), 1767296965000)]
    #[case(TimeFormat::UnixTimestamp("17672969655"), 17672969655)]
    #[case(TimeFormat::UnixTimestamp("176729696559"), 176729696559)]
    #[case(TimeFormat::UnixTimestamp("1767296965592"), 1767296965592)]
    fn should_parse_time_format_to_system_time(
        #[case] time_format: TimeFormat,
        #[case] expected_timestamp_millis: u128,
    ) {
        // Arrange

        // Act
        let system_time = time_format.parse_system_time();

        // Assert
        assert!(system_time.is_ok());
        let system_time = system_time.expect("Must be valid System Time at this point.");
        let duration_since_unix_epoch = system_time.duration_since(SystemTime::UNIX_EPOCH);
        assert!(duration_since_unix_epoch.is_ok());
        let timestamp_millis = duration_since_unix_epoch
            .expect("Must be valid Duration at this point.")
            .as_millis();
        assert_eq!(timestamp_millis, expected_timestamp_millis)
    }

    #[rstest]
    #[case(TimeFormat::InvalidFormat, TimeFormatError::InvalidFormat)]
    #[case(
        TimeFormat::Rfc3339("2026-01-32T12:00:00Z"),
        TimeFormatError::InvalidFormat
    )]
    #[case(
        TimeFormat::Rfc3339("2026-13-01T12:00:00Z"),
        TimeFormatError::InvalidFormat
    )]
    #[case(
        TimeFormat::Rfc3339("2026-01-01T25:00:00Z"),
        TimeFormatError::InvalidFormat
    )]
    #[case(
        TimeFormat::Rfc3339("2026-01-01T12:60:00Z"),
        TimeFormatError::InvalidFormat
    )]
    #[case(
        TimeFormat::Rfc3339("2026-01-01T12:00:60Z"),
        TimeFormatError::InvalidFormat
    )]
    fn should_error_when_parsing_non_valid_time_format(
        #[case] time_format: TimeFormat,
        #[case] expected_error: TimeFormatError,
    ) {
        // Arrange

        // Act
        let system_time = time_format.parse_system_time();

        // Assert
        assert!(system_time.is_err());
        let error = system_time.expect_err("Must not be valid System Time at this point.");
        assert_eq!(error, expected_error);
    }
}
