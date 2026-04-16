use clap::Parser;
use clap_stdin::FileOrStdin;
use kitlog::process_log;
use std::process;

/// A utility to parse and visually format logs.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "PATH", default_value = "-")]
    /// Path to a log file
    input: FileOrStdin,
}

fn main() {
    let args = Cli::parse();

    match args.input.into_reader() {
        Ok(reader) => {
            if let Err(e) = process_log(reader) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to open input: {}", e);
            process::exit(1);
        }
    }
}
