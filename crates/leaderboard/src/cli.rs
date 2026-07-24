use crate::errors::GameError;
use crate::local_file::LocalFileLeaderboard;
use crate::models::{LeaderboardEntry, SortBy};
use crate::trait_def::Leaderboard;
use clap::Parser;
use std::path::PathBuf;

/// Command line interface options for the Leaderboard application.
#[derive(Parser, Debug)]
#[command(name = "leaderboard")]
#[command(about = "Display and manage the global puzzle leaderboard")]
pub struct CliArgs {
    /// Display the leaderboard
    #[arg(long)]
    pub leaderboard: bool,

    /// Sorting criteria (score or recency)
    #[arg(long, default_value = "score")]
    pub sort: String,

    /// Number of top entries to display
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Custom file path for the local leaderboard database
    #[arg(long)]
    pub file: Option<PathBuf>,
}

/// Formats and displays leaderboard entries as a CLI table.
pub fn format_leaderboard_table(entries: &[LeaderboardEntry]) -> String {
    if entries.is_empty() {
        return "No leaderboard entries found.".to_string();
    }

    let mut out = String::new();
    out.push_str(&format!(
        "{:<6} {:<20} {:<20} {:<10} {:<20}\n",
        "Rank", "Player Name", "Puzzle Type", "Score", "Timestamp"
    ));
    out.push_str(&format!("{}\n", "-".repeat(78)));

    for (idx, entry) in entries.iter().enumerate() {
        out.push_str(&format!(
            "{:<6} {:<20} {:<20} {:<10} {:<20}\n",
            idx + 1,
            entry.player_name,
            entry.puzzle_type,
            entry.score,
            entry.timestamp
        ));
    }

    out
}

/// Executes the CLI command with the given arguments and leaderboard implementation.
pub fn run_cli_with_args<L: Leaderboard>(args: &CliArgs, leaderboard: &L) -> Result<Option<String>, GameError> {
    if args.leaderboard {
        let sort_by: SortBy = args.sort.parse().map_err(|e| GameError::Config(e))?;
        let top_entries = leaderboard
            .get_top_entries(args.limit, sort_by)?;

        let output = format_leaderboard_table(&top_entries);
        Ok(Some(output))
    } else {
        Ok(None)
    }
}

/// Runs the default CLI workflow loading entries from `LocalFileLeaderboard`.
pub fn run_cli(args: &CliArgs) -> Result<Option<String>, GameError> {
    let path = args.file.clone().unwrap_or_else(LocalFileLeaderboard::default_path);
    let leaderboard = LocalFileLeaderboard::new(path);
    run_cli_with_args(args, &leaderboard)
}
