use ulid::Ulid;
use ulidgen::{ParseSystemTime, TimeFormat, TimeFormatError};

const INVALID_ARGS_ERROR_MESSAGE: &str = "Invalid arguments.";
const INVALID_TIME_ERROR_MESSAGE: &str = "Invalid time format.";
const HELP_MESSAGE: &str = r#"USAGE:
    ulidgen [OPTIONS]

OPTIONS:
    -t, --time <TIME>
        Generate time based ULID. TIME must be one of the following formats:

        - Unix Timestamp
          - Digits only
          - Up to 10 digits: seconds since Unix Epoch
          - From 11 to 13 digits: milliseconds since Unix Epoch

        - RFC 3339
          - Timezone offsets are supported

        - Date only
          - Format: YYYY-MM-DD
          - Interpreted as midnight UTC

    -h, --help
        Print help message

    -v, --version
        Print version information

EXAMPLES:
    Generate ULID for right now
      ulidgen

    Generate ULID for Unix Timestamp
      ulidgen -t 1767270896
      ulidgen -t 1767270896000

    Generate ULID for RFC 3339
      ulidgen -t 2026-01-01T12:34:56Z
      ulidgen -t 2026-01-01T12:34:56.789-03:00

    Generate ULID for Date Only
      ulidgen -t 2026-01-01
"#;

#[derive(Debug, PartialEq)]
enum RunError {
    InvalidArgs,
    InvalidTimeFormat,
}

impl From<TimeFormatError> for RunError {
    fn from(_value: TimeFormatError) -> Self {
        Self::InvalidTimeFormat
    }
}

fn run(args: &[String]) -> Result<String, RunError> {
    match args {
        [_, opt] => match opt.as_str() {
            "-v" | "--version" => Ok(env!("CARGO_PKG_VERSION").to_string()),
            "-h" | "--help" => Ok(HELP_MESSAGE.to_string()),
            _ => Err(RunError::InvalidArgs),
        },
        [_, opt, parameter] => match opt.as_str() {
            "-t" | "--time" => Ok(Ulid::from_datetime(
                TimeFormat::new(parameter).parse_system_time()?,
            )
            .to_string()),
            _ => Err(RunError::InvalidArgs),
        },
        [_] => Ok(Ulid::new().to_string()),
        _ => Err(RunError::InvalidArgs),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = run(args.as_slice());

    match result {
        Ok(v) => println!("{}", v),
        Err(RunError::InvalidArgs) => {
            eprintln!("{}\n", INVALID_ARGS_ERROR_MESSAGE);
            eprintln!("{}", HELP_MESSAGE);
            std::process::exit(1);
        }
        Err(RunError::InvalidTimeFormat) => {
            eprintln!("{}\n", INVALID_TIME_ERROR_MESSAGE);
            eprintln!("{}", HELP_MESSAGE)
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(vec!["ulidgen".to_string(), "-v".to_string()])]
    #[case(vec!["ulidgen".to_string(), "--version".to_string()])]
    fn should_return_version(#[case] args: Vec<String>) {
        // Arrange
        let args = args.as_slice();

        let expected_version = env!("CARGO_PKG_VERSION");

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_ok());
        let version = result.expect("Must be valid at this point.");
        assert_eq!(version, expected_version);
    }

    #[rstest]
    #[case(vec!["ulidgen".to_string(), "-h".to_string()])]
    #[case(vec!["ulidgen".to_string(), "--help".to_string()])]
    fn should_return_help_message(#[case] args: Vec<String>) {
        // Arrange
        let args = args.as_slice();

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_ok());
        let help = result.expect("Must be valid at this point.");
        assert_eq!(help, HELP_MESSAGE);
    }

    #[rstest]
    #[case(vec!["ulidgen".to_string(), "-w".to_string()])]
    #[case(vec!["ulidgen".to_string(), "--whatever".to_string()])]
    #[case(vec!["ulidgen".to_string(), "-w".to_string(), "value".to_string()])]
    #[case(vec!["ulidgen".to_string(), "--whatever".to_string(), "value".to_string()])]
    #[case(vec!["ulidgen".to_string(), "-w".to_string(), "value".to_string(), "-x".to_string()])]
    #[case(vec!["ulidgen".to_string(), "--whatever".to_string(), "value".to_string(), "-x".to_string()])]
    fn should_return_error_for_unknown_args(#[case] args: Vec<String>) {
        // Arrange
        let args = args.as_slice();

        let expected_error = RunError::InvalidArgs;

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_err());
        let error = result.expect_err("Must not be valid at this point.");
        assert_eq!(error, expected_error);
    }

    #[rstest]
    #[case(vec!["ulidgen".to_string(), "-t".to_string(), "1767270896".to_string()], 1767270896000)]
    #[case(vec!["ulidgen".to_string(), "--time".to_string(), "1767270896".to_string()], 1767270896000)]
    #[case(vec!["ulidgen".to_string(), "-t".to_string(), "1767270896123".to_string()], 1767270896123)]
    #[case(vec!["ulidgen".to_string(), "--time".to_string(), "1767270896123".to_string()], 1767270896123)]
    #[case(vec!["ulidgen".to_string(), "-t".to_string(), "2026-01-01T12:34:56Z".to_string()], 1767270896000)]
    #[case(vec!["ulidgen".to_string(), "--time".to_string(), "2026-01-01T12:34:56Z".to_string()], 1767270896000)]
    #[case(vec!["ulidgen".to_string(), "-t".to_string(), "2026-01-01T12:34:56.789-03:00".to_string()], 1767281696789)]
    #[case(vec!["ulidgen".to_string(), "--time".to_string(), "2026-01-01T12:34:56.789-03:00".to_string()], 1767281696789)]
    #[case(vec!["ulidgen".to_string(), "-t".to_string(), "2026-01-01".to_string()], 1767225600000)]
    #[case(vec!["ulidgen".to_string(), "--time".to_string(), "2026-01-01".to_string()], 1767225600000)]
    fn should_return_ulid_with_time(#[case] args: Vec<String>, #[case] expected_timestamp_ms: u64) {
        // Arrange
        let args = args.as_slice();

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_ok());
        let timestamp = Ulid::from_string(&result.expect("Must be valid at this point."))
            .expect("Must be valid ULID at this point.")
            .timestamp_ms();
        assert_eq!(timestamp, expected_timestamp_ms);
    }

    #[test]
    fn should_return_invalid_time_format() {
        // Arrange
        let args = vec![
            "ulidgen".to_string(),
            "-t".to_string(),
            "2026-01".to_string(),
        ];
        let args = args.as_slice();

        let expected_error = RunError::InvalidTimeFormat;

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_err());
        let error = result.expect_err("Must not be valid at this point.");
        assert_eq!(error, expected_error);
    }

    #[test]
    fn should_return_ulid() {
        // Arrange
        let args = vec!["ulidgen".to_string()];
        let args = args.as_slice();

        // Act
        let result = run(args);

        // Assert
        assert!(result.is_ok());
        let ulid = Ulid::from_string(&result.expect("Must be valid at this point."));
        assert!(ulid.is_ok());
    }
}
