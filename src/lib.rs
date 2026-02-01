use regex::Regex;
use std::io::{self, BufRead, BufWriter, Read, Write};

pub struct LogLine {
    pub scaling: u8,
    pub variant: &'static str,
    pub color: &'static str,
}

impl LogLine {
    pub fn format_header(&self, header: &str) -> String {
        format!(
            "\x1b[{}m\x1b]66;s={};{}\x07\x1b[0m", // Changed here
            self.color, self.scaling, header
        )
    }

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

    pub fn trailing_newlines(&self) -> String {
        "\n".repeat(self.scaling as usize)
    }
}

pub enum LogLevel {
    Error { line: LogLine },
    Warn { line: LogLine },
    Info { line: LogLine },
    Debug { line: LogLine },
    Trace { line: LogLine },
}

impl LogLevel {
    pub fn new_error() -> Self {
        Self::Error {
            line: LogLine {
                scaling: 5,
                variant: "error",
                color: "1;31",
            },
        }
    }
    pub fn new_warn() -> Self {
        Self::Warn {
            line: LogLine {
                scaling: 4,
                variant: "warn",
                color: "1;33",
            },
        }
    }
    pub fn new_info() -> Self {
        Self::Info {
            line: LogLine {
                scaling: 3,
                variant: "info",
                color: "1;34",
            },
        }
    }
    pub fn new_debug() -> Self {
        Self::Debug {
            line: LogLine {
                scaling: 2,
                variant: "debug",
                color: "1;32",
            },
        }
    }
    pub fn new_trace() -> Self {
        Self::Trace {
            line: LogLine {
                scaling: 1,
                variant: "trace",
                color: "1;30",
            },
        }
    }

    pub fn get_log_line(&self) -> &LogLine {
        match self {
            Self::Error { line }
            | Self::Warn { line }
            | Self::Info { line }
            | Self::Debug { line }
            | Self::Trace { line } => line,
        }
    }
}

fn build_regex_pattern() -> String {
    let variants = [
        LogLevel::new_error().get_log_line().variant,
        LogLevel::new_warn().get_log_line().variant,
        LogLevel::new_info().get_log_line().variant,
        LogLevel::new_debug().get_log_line().variant,
        LogLevel::new_trace().get_log_line().variant,
    ];

    format!(r"(?i)^.*?({}).*?\s", variants.join("|"))
}

fn from_keyword(keyword: &str) -> Option<LogLevel> {
    match keyword {
        "error" => Some(LogLevel::new_error()),
        "warn" => Some(LogLevel::new_warn()),
        "info" => Some(LogLevel::new_info()),
        "debug" => Some(LogLevel::new_debug()),
        "trace" => Some(LogLevel::new_trace()),
        _ => None,
    }
}

fn parse_line(line: &str, re: &Regex) -> Option<(usize, LogLevel)> {
    let caps = re.captures(line)?;
    let keyword = caps.get(1)?.as_str().to_lowercase();
    let full_match = caps.get(0)?;

    let level = from_keyword(&keyword)?;
    Some((full_match.end(), level))
}

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
                let log = level.get_log_line();

                write!(handle, "{}", log.format_header(header))?;
                write!(handle, "{}", log.format_message(message))?;
                write!(handle, "{}", log.trailing_newlines())?;
            }
            None => {
                writeln!(handle, "{}", line)?;
            }
        }
    }

    handle.flush()?;
    Ok(())
}
