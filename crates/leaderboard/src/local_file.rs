use crate::models::{LeaderboardEntry, LeaderboardError, SortBy};
use crate::trait_def::Leaderboard;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// A local file-backed leaderboard implementation that persists score entries to disk.
pub struct LocalFileLeaderboard {
    file_path: PathBuf,
}

impl LocalFileLeaderboard {
    /// Creates a new `LocalFileLeaderboard` using the specified file path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            file_path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns the default system storage path (`~/.mesh/leaderboard.json` or fallback `./leaderboard.json`).
    pub fn default_path() -> PathBuf {
        if let Some(mut dir) = dirs_next() {
            dir.push(".mesh");
            dir.push("leaderboard.json");
            dir
        } else {
            PathBuf::from("./leaderboard.json")
        }
    }

    /// Creates a `LocalFileLeaderboard` instance with the default file path.
    pub fn with_default_path() -> Self {
        Self::new(Self::default_path())
    }

    /// Returns the file path of this leaderboard instance.
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    fn read_entries(&self) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path)?;
        if file.metadata()?.len() == 0 {
            return Ok(Vec::new());
        }
        let reader = BufReader::new(file);
        let entries: Vec<LeaderboardEntry> = serde_json::from_reader(reader)
            .map_err(|e| LeaderboardError::Persistence(format!("Failed to deserialize leaderboard: {}", e)))?;
        Ok(entries)
    }

    fn write_entries(&self, entries: &[LeaderboardEntry]) -> Result<(), LeaderboardError> {
        if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = File::create(&self.file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, entries)?;
        Ok(())
    }

    /// Sorts entries according to `SortBy`.
    pub fn sort_entries(entries: &mut [LeaderboardEntry], sort_by: SortBy) {
        match sort_by {
            SortBy::Score => {
                entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.timestamp.cmp(&a.timestamp)));
            }
            SortBy::Recency => {
                entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| b.score.cmp(&a.score)));
            }
        }
    }
}

fn dirs_next() -> Option<PathBuf> {
    #[allow(deprecated)]
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

impl Leaderboard for LocalFileLeaderboard {
    fn add_entry(&mut self, entry: LeaderboardEntry) -> Result<(), LeaderboardError> {
        let mut entries = self.read_entries()?;
        entries.push(entry);
        self.write_entries(&entries)
    }

    fn get_top_entries(&self, limit: usize, sort_by: SortBy) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        let mut entries = self.read_entries()?;
        Self::sort_entries(&mut entries, sort_by);
        entries.truncate(limit);
        Ok(entries)
    }

    fn get_all_entries(&self) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        self.read_entries()
    }
}
