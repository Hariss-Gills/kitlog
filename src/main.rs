use clap::Parser;

/// Print log files using kitty text-sizing protocol.
#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path).expect("could not read file");
    for line in content.lines() {
        print!("\x1b]66;s=2;{}\x07\n\n", line);
    }
}
