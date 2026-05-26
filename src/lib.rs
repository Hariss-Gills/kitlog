//! Log parsing and terminal rendering utilities.
//!
//! Parses line-based logs, detects log levels, and renders styled output using
//! ANSI escape codes and Kitty’s OSC 66 text sizing protocol.
//!
//! Unknown lines are passed through unchanged.
pub mod config;
pub use crate::config::Config;
pub use crate::config::Level;
pub use crate::config::Levels;
use regex::Regex;
use std::io::{self, BufRead, BufWriter, Read, Write};

/// Constructs a case-insensitive regex pattern from configured level keywords.
fn build_regex_pattern(config: &Config) -> String {
    format!(
        r"(?i)^.*?({}|{}|{}|{}|{}).*?\s",
        config.levels.error.keyword,
        config.levels.warn.keyword,
        config.levels.info.keyword,
        config.levels.debug.keyword,
        config.levels.trace.keyword,
    )
}

/// Parses a single text line to identify its log level and header boundary.
///
/// # Arguments
///
/// * `line` - The raw string slice of a single log line.
/// * `re` - A compiled regular expression matching log level keywords.
/// * `levels` - The configured log level definitions to match against.
///
/// # Returns
///
/// Returns an `Option` containing a tuple of:
/// 1. `usize`: The byte index representing the end of the header (where the message begins).
/// 2. `&Level`: The identified severity level.
///
/// Returns `None` if no recognized log level is found.
fn parse_line<'a>(line: &str, re: &Regex, levels: &'a Levels) -> Option<(usize, &'a Level)> {
    let caps = re.captures(line)?;
    let keyword = caps.get(1)?.as_str().to_lowercase();
    let full_match = caps.get(0)?;
    let level = levels.by_keyword(&keyword)?;
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
/// * `config` - The configuration specifying log level keywords, colors, and text scaling.
///
/// # Errors
///
/// Returns an error if the internal regex fails to compile, or if an I/O error
/// occurs while reading from the provided reader or writing to `stdout`.
pub fn process_log<R: Read>(reader: R, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new(&build_regex_pattern(&config))?;
    let reader = io::BufReader::new(reader);
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    for line_result in reader.lines() {
        let line = line_result?;
        match parse_line(&line, &re, &config.levels) {
            Some((end, level)) => {
                let header = line[..end].trim_end();
                let message = line[end..].trim();

                write!(handle, "{}", level.format_header(header))?;
                write!(handle, "{}", level.format_message(message))?;
                write!(handle, "{}", level.trailing_newlines())?;
            }
            None => {
                writeln!(handle, "{}", line)?;
            }
        }
    }

    handle.flush()?;
    Ok(())
}
