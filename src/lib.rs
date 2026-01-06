use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Error = 5,
    Warn = 4,
    Info = 3,
    Debug = 2,
    Trace = 1,
}

impl LogLevel {
    pub fn scaling(&self) -> u8 {
        *self as u8
    }

    pub fn build_regex_pattern() -> String {
        let variants: Vec<String> = LogLevel::iter().map(|v| v.to_string()).collect();
        format!(r"(?i)^.*?({}).*?\s", variants.join("|"))
    }

    pub fn parse_line(line: &str, re: &Regex) -> Option<(usize, LogLevel)> {
        re.captures(line).and_then(|caps| {
            let keyword = caps.get(1)?.as_str();
            let level = LogLevel::from_str(&keyword.to_lowercase()).ok()?;
            let full_match = caps.get(0)?;
            Some((full_match.end(), level))
        })
    }

    pub fn format_header(header: &str, level: LogLevel) -> String {
        format!("\x1b]66;s={};{}\x07", level.scaling(), header)
    }

    pub fn format_message(message: &str, level: LogLevel) -> String {
        let scale = level.scaling();
        let chunk_size = scale as usize;
        let chars: Vec<char> = message.chars().collect();

        let mut result = String::new();
        for chunk in chars.chunks(chunk_size) {
            let chunk_str: String = chunk.iter().collect();
            result.push_str(&format!(
                "\x1b]66;s={}:w=1:n=1:d={}:v=2;{}\x07",
                scale, scale, chunk_str
            ));
        }
        result
    }

    pub fn trailing_newlines(level: LogLevel) -> String {
        let scale = level.scaling();
        if scale > 1 {
            "\n".repeat((scale) as usize)
        } else {
            String::new()
        }
    }
}

pub fn process_log_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new(&LogLevel::build_regex_pattern())?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    for line_result in reader.lines() {
        let line = line_result?;
        match LogLevel::parse_line(&line, &re) {
            Some((end, level)) => {
                let header = line[..end].trim_end();
                let message = line[end..].trim();
                // Fast: Writing to memory buffer
                write!(handle, "{}", LogLevel::format_header(header, level))?;
                write!(handle, "{}", LogLevel::format_message(message, level))?;
                write!(handle, "{}", LogLevel::trailing_newlines(level))?;
            }
            None => writeln!(handle, "{}", line)?,
        }
    }
    handle.flush()?;
    Ok(())
}
