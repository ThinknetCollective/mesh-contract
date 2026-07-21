use crate::models::{LeaderboardEntry, LeaderboardError, SortBy};
use crate::trait_def::Leaderboard;

/// A stub implementation of a remote, HTTP-backed leaderboard.
/// Demonstrates that callers using the `Leaderboard` trait can seamlessly switch backends.
pub struct RemoteLeaderboard {
    endpoint_url: String,
    in_memory_cache: Vec<LeaderboardEntry>,
}

impl RemoteLeaderboard {
    /// Creates a new `RemoteLeaderboard` instance configured with an endpoint URL.
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
            in_memory_cache: Vec::new(),
        }
    }

    /// Returns the endpoint URL.
    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }
}

impl Leaderboard for RemoteLeaderboard {
    fn add_entry(&mut self, entry: LeaderboardEntry) -> Result<(), LeaderboardError> {
        // In a real implementation, this would send an HTTP POST request to endpoint_url.
        self.in_memory_cache.push(entry);
        Ok(())
    }

    fn get_top_entries(&self, limit: usize, sort_by: SortBy) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        let mut entries = self.in_memory_cache.clone();
        match sort_by {
            SortBy::Score => {
                entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.timestamp.cmp(&a.timestamp)));
            }
            SortBy::Recency => {
                entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| b.score.cmp(&a.score)));
            }
        }
        entries.truncate(limit);
        Ok(entries)
    }

    fn get_all_entries(&self) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        Ok(self.in_memory_cache.clone())
    }
}
