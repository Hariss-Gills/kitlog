use clap::Parser;
use clap_stdin::FileOrStdin;
use kitlog::Config;
use kitlog::process_log;
use std::path::PathBuf;
use std::process;

/// A utility to parse and visually format logs.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "PATH", default_value = "-")]
    /// Path to a log file
    input: FileOrStdin,
    /// Optional Path to config file
    #[clap(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();

    match args.input.into_reader() {
        Ok(reader) => {
            let config: Config = match args.config {
                Some(path) => {
                    confy::load_path(path).expect("Failed to load config from specified path")
                }
                None => confy::load("kitlog", "config")
                    .expect("Config file could not be created or loaded"),
            };

            if let Err(e) = process_log(reader, config) {
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
