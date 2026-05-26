//! Configuration types for log level definitions, keywords, colors, and text scaling.
//!
//! Provides [`Config`], [`Levels`], and [`Level`] structs with serialization support
//! and sensible defaults for common log severity levels.

use serde::{Deserialize, Serialize};

/// Top-level configuration for log parsing and rendering.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub levels: Levels,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            levels: Levels {
                error: Level {
                    scaling: 5,
                    color: "1;31".into(),
                    keyword: "error".into(),
                },
                warn: Level {
                    scaling: 4,
                    color: "1;33".into(),
                    keyword: "warn".into(),
                },
                info: Level {
                    scaling: 3,
                    color: "1;34".into(),
                    keyword: "info".into(),
                },
                debug: Level {
                    scaling: 2,
                    color: "1;32".into(),
                    keyword: "debug".into(),
                },
                trace: Level {
                    scaling: 1,
                    color: "1;30".into(),
                    keyword: "trace".into(),
                },
            },
        }
    }
}

/// A set of named log severity levels with their associated styling.
#[derive(Debug, Serialize, Deserialize)]
pub struct Levels {
    pub error: Level,
    pub warn: Level,
    pub info: Level,
    pub debug: Level,
    pub trace: Level,
}

impl Levels {
    /// Looks up a log level by its configured keyword string.
    ///
    /// # Arguments
    ///
    /// * `keyword` - The keyword to search for (case-sensitive).
    ///
    /// # Returns
    ///
    /// Returns `Some(&Level)` if a match is found, or `None` otherwise.
    pub fn by_keyword(&self, keyword: &str) -> Option<&Level> {
        [
            &self.error,
            &self.warn,
            &self.info,
            &self.debug,
            &self.trace,
        ]
        .into_iter()
        .find(|level| level.keyword == keyword)
    }
}

/// A single log severity level with its display properties.
#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
    pub scaling: u8,
    pub color: String,
    pub keyword: String,
}

impl Level {
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
