use crate::models::{LeaderboardEntry, LeaderboardError, SortBy};

/// Core trait defining operations for leaderboard storage and retrieval.
/// Designed to support both local caching (`LocalFileLeaderboard`) and future network backends (`RemoteLeaderboard`).
pub trait Leaderboard {
    /// Inserts a new score entry into the leaderboard.
    fn add_entry(&mut self, entry: LeaderboardEntry) -> Result<(), LeaderboardError>;

    /// Retrieves top N entries sorted by the specified criteria (`SortBy::Score` or `SortBy::Recency`).
    fn get_top_entries(&self, limit: usize, sort_by: SortBy) -> Result<Vec<LeaderboardEntry>, LeaderboardError>;

    /// Retrieves all recorded entries.
    fn get_all_entries(&self) -> Result<Vec<LeaderboardEntry>, LeaderboardError>;
}
