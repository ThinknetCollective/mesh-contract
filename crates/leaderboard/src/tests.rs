use super::*;
use crate::cli::{format_leaderboard_table, run_cli_with_args, CliArgs};
use tempfile::NamedTempFile;

#[test]
fn test_entry_creation() {
    let entry = LeaderboardEntry::new("Alice", "Sudoku", 950, 1600000000);
    assert_eq!(entry.player_name, "Alice");
    assert_eq!(entry.puzzle_type, "Sudoku");
    assert_eq!(entry.score, 950);
    assert_eq!(entry.timestamp, 1600000000);
}

#[test]
fn test_insertion_and_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_path_buf();

    {
        let mut leaderboard = LocalFileLeaderboard::new(&file_path);
        let entry1 = LeaderboardEntry::new("Alice", "Crossword", 500, 100);
        let entry2 = LeaderboardEntry::new("Bob", "Crossword", 800, 200);

        leaderboard.add_entry(entry1).unwrap();
        leaderboard.add_entry(entry2).unwrap();
    }

    // Reload from file in a new instance to verify persistence across sessions
    let leaderboard = LocalFileLeaderboard::new(&file_path);
    let entries = leaderboard.get_all_entries().unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].player_name, "Alice");
    assert_eq!(entries[1].player_name, "Bob");
}

#[test]
fn test_sorting_by_score() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut leaderboard = LocalFileLeaderboard::new(temp_file.path());

    leaderboard.add_entry(LeaderboardEntry::new("Player1", "Math", 100, 1000)).unwrap();
    leaderboard.add_entry(LeaderboardEntry::new("Player2", "Math", 300, 1000)).unwrap();
    leaderboard.add_entry(LeaderboardEntry::new("Player3", "Math", 200, 1000)).unwrap();

    let top = leaderboard.get_top_entries(10, SortBy::Score).unwrap();

    assert_eq!(top.len(), 3);
    assert_eq!(top[0].player_name, "Player2"); // Score 300
    assert_eq!(top[1].player_name, "Player3"); // Score 200
    assert_eq!(top[2].player_name, "Player1"); // Score 100
}

#[test]
fn test_sorting_by_recency() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut leaderboard = LocalFileLeaderboard::new(temp_file.path());

    leaderboard.add_entry(LeaderboardEntry::new("Old", "Logic", 500, 1000)).unwrap();
    leaderboard.add_entry(LeaderboardEntry::new("Newest", "Logic", 100, 3000)).unwrap();
    leaderboard.add_entry(LeaderboardEntry::new("Medium", "Logic", 900, 2000)).unwrap();

    let top = leaderboard.get_top_entries(10, SortBy::Recency).unwrap();

    assert_eq!(top.len(), 3);
    assert_eq!(top[0].player_name, "Newest"); // Timestamp 3000
    assert_eq!(top[1].player_name, "Medium"); // Timestamp 2000
    assert_eq!(top[2].player_name, "Old");    // Timestamp 1000
}

#[test]
fn test_top_n_retrieval() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut leaderboard = LocalFileLeaderboard::new(temp_file.path());

    for i in 1..=15 {
        leaderboard
            .add_entry(LeaderboardEntry::new(
                format!("Player{}", i),
                "Puzzle",
                i * 10,
                1000 + i,
            ))
            .unwrap();
    }

    // Default --leaderboard displays top 10 entries
    let top10 = leaderboard.get_top_entries(10, SortBy::Score).unwrap();
    assert_eq!(top10.len(), 10);
    assert_eq!(top10[0].player_name, "Player15");
    assert_eq!(top10[0].score, 150);
    assert_eq!(top10[9].player_name, "Player6");
    assert_eq!(top10[9].score, 60);
}

#[test]
fn test_remote_leaderboard_trait_compatibility() {
    fn add_score<L: Leaderboard>(leaderboard: &mut L) {
        leaderboard
            .add_entry(LeaderboardEntry::new("RemotePlayer", "Network", 999, 5000))
            .unwrap();
    }

    let mut remote = RemoteLeaderboard::new("https://api.leaderboard.example.com");
    add_score(&mut remote);

    let top = remote.get_top_entries(10, SortBy::Score).unwrap();
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].player_name, "RemotePlayer");
    assert_eq!(remote.endpoint_url(), "https://api.leaderboard.example.com");
}

#[test]
fn test_cli_formatting_and_execution() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut leaderboard = LocalFileLeaderboard::new(temp_file.path());

    leaderboard
        .add_entry(LeaderboardEntry::new("Champ", "Wordle", 1000, 1700000000))
        .unwrap();

    let args = CliArgs {
        leaderboard: true,
        sort: "score".to_string(),
        limit: 10,
        file: Some(temp_file.path().to_path_buf()),
    };

    let result = run_cli_with_args(&args, &leaderboard).unwrap();
    assert!(result.is_some());
    let output = result.unwrap();

    assert!(output.contains("Champ"));
    assert!(output.contains("Wordle"));
    assert!(output.contains("1000"));

    let table_empty = format_leaderboard_table(&[]);
    assert_eq!(table_empty, "No leaderboard entries found.");
}
