use clap::Parser;
use leaderboard::cli::{run_cli, CliArgs};

fn main() {
    let args = CliArgs::parse();
    match run_cli(&args) {
        Ok(Some(output)) => println!("{}", output),
        Ok(None) => {
            println!("Use --leaderboard to display the top scores.");
        }
        Err(err) => {
            // Cleanly print readable error strings to standard error channels without backtraces
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
