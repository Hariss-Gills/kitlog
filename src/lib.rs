//! Log parsing and terminal rendering utilities.
//!
//! Parses line-based logs, detects log levels, and renders styled output using
//! ANSI escape codes and Kitty’s OSC 66 text sizing protocol.
//!
//! Unknown lines are passed through unchanged.

use regex::Regex;
use std::io::{self, BufRead, BufWriter, Read, Write};
use std::str::FromStr;

/// Defines the formatting attributes and metadata for a specific log level.
///
/// This struct holds the configuration needed to style both the header and
/// the message body of a matched log line.
pub struct LogLine {
    /// A numeric multiplier used for chunking messages and appending newlines.
    pub scaling: u8,
    /// The string representation of the log level (e.g., "error", "info").
    pub variant: &'static str,
    /// The ANSI color code string (e.g., "1;31" for bold red).
    pub color: &'static str,
}

impl LogLine {
    /// Formats the header portion of a log line.
    ///
    /// Applies the assigned ANSI color and injects the text sizing escape
    /// sequence (`\x1b]66;...`) to denote the log's scaling and header boundaries.
    ///
    /// # Arguments
    ///
    /// * `header` - The prefix of the log line containing timestamps, origins, and the log level.
    pub fn format_header(&self, header: &str) -> String {
        format!(
            "\x1b[{}m\x1b]66;s={};{}\x07\x1b[0m",
            self.color, self.scaling, header
        )
    }

    /// Formats the message payload of a log line.
    ///
    /// Splits the message into chunks based on the `scaling` factor and wraps
    /// each chunk in the the text sizing escape sequence.
    ///
    /// # Arguments
    ///
    /// * `message` - The actual log message body, stripped of its header.
    pub fn format_message(&self, message: &str) -> String {
        let chunk_size = self.scaling as usize;
        let chars: Vec<char> = message.chars().collect();

        let mut result = String::new();
        for chunk in chars.chunks(chunk_size) {
            let chunk_str: String = chunk.iter().collect();
            result.push_str(&format!(
                "\x1b[{}m\x1b]66;s={}:w=1:n=1:d={}:v=2;{}\x07\x1b[0m",
                self.color, self.scaling, self.scaling, chunk_str
            ));
        }
        result
    }

    /// Generates trailing newlines corresponding to the log level's scaling factor.
    ///
    /// Higher severity logs typically have a higher scaling factor, resulting
    /// in more vertical spacing after the log entry.
    pub fn trailing_newlines(&self) -> String {
        "\n".repeat(self.scaling as usize)
    }
}

/// Supported log levels in the input stream.
pub enum LogLevel {
    /// Error-level log output.
    Error,
    /// Warning-level log output.
    Warn,
    /// Informational log output.
    Info,
    /// Debug-level log output.
    Debug,
    /// Trace-level log output.
    Trace,
}

impl LogLevel {
    /// Returns the static formatting attributes associated with the log level variant.
    ///
    /// - `Error`: Bold Red, Scaling 5
    /// - `Warn`: Bold Yellow, Scaling 4
    /// - `Info`: Bold Blue, Scaling 3
    /// - `Debug`: Bold Green, Scaling 2
    /// - `Trace`: Bold Black/Gray, Scaling 1
    pub fn attributes(&self) -> LogLine {
        match self {
            Self::Error => LogLine {
                scaling: 5,
                variant: "error",
                color: "1;31",
            },
            Self::Warn => LogLine {
                scaling: 4,
                variant: "warn",
                color: "1;33",
            },
            Self::Info => LogLine {
                scaling: 3,
                variant: "info",
                color: "1;34",
            },
            Self::Debug => LogLine {
                scaling: 2,
                variant: "debug",
                color: "1;32",
            },
            Self::Trace => LogLine {
                scaling: 1,
                variant: "trace",
                color: "1;30",
            },
        }
    }
}

/// Parses a string slice to determine the corresponding `LogLevel`.
///
/// This conversion is case-insensitive.
impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Unknown log level: {}", s)),
        }
    }
}

/// Constructs a regular expression pattern to detect log level keywords.
///
/// The pattern is case-insensitive and lazily matches any text up to one
/// of the standard log variants, followed by trailing whitespace.
fn build_regex_pattern() -> String {
    let variants = [
        LogLevel::Error.attributes().variant,
        LogLevel::Warn.attributes().variant,
        LogLevel::Info.attributes().variant,
        LogLevel::Debug.attributes().variant,
        LogLevel::Trace.attributes().variant,
    ];

    format!(r"(?i)^.*?({}).*?\s", variants.join("|"))
}

/// Parses a single text line to identify its log level and header boundary.
///
/// # Arguments
///
/// * `line` - The raw string slice of a single log line.
/// * `re` - A compiled regular expression matching log level keywords.
///
/// # Returns
///
/// Returns an `Option` containing a tuple of:
/// 1. `usize`: The byte index representing the end of the header (where the message begins).
/// 2. `LogLevel`: The identified severity level.
///
/// Returns `None` if no recognized log level is found.
fn parse_line(line: &str, re: &Regex) -> Option<(usize, LogLevel)> {
    let caps = re.captures(line)?;
    let keyword = caps.get(1)?.as_str().to_lowercase();
    let full_match = caps.get(0)?;

    let level = LogLevel::from_str(&keyword).ok()?;
    Some((full_match.end(), level))
}

/// Processes an input stream of log lines, formats them, and writes to standard output.
///
/// Lines containing a recognized log level are split into header and message,
/// styled with terminal escape sequences, and padded with newlines.
/// Unrecognized lines are passed through unmodified.
///
/// # Arguments
///
/// * `reader` - Any type implementing the `Read` trait (e.g., `std::fs::File`, `std::io::stdin`).
///
/// # Errors
///
/// Returns an error if the internal regex fails to compile, or if an I/O error
/// occurs while reading from the provided reader or writing to `stdout`.
pub fn process_log<R: Read>(reader: R) -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new(&build_regex_pattern())?;
    let reader = io::BufReader::new(reader);

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    for line_result in reader.lines() {
        let line = line_result?;
        match parse_line(&line, &re) {
            Some((end, level)) => {
                let header = line[..end].trim_end();
                let message = line[end..].trim();
                let log_line = level.attributes();

                write!(handle, "{}", log_line.format_header(header))?;
                write!(handle, "{}", log_line.format_message(message))?;
                write!(handle, "{}", log_line.trailing_newlines())?;
            }
            None => {
                writeln!(handle, "{}", line)?;
            }
        }
    }

    handle.flush()?;
    Ok(())
}
