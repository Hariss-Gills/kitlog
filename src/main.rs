use clap::Parser;
use kitlog::process_log_file;
use std::path;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    if let Err(e) = process_log_file(&args.path) {
        eprintln!("Error processing log file: {}", e);
        process::exit(1);
    }
}
