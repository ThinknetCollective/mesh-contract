/// Maximum possible score a contributor can achieve.
pub const MAX_SCORE: u64 = 1000;

/// Minimum possible score (floored at 0).
pub const MIN_SCORE: u64 = 0;

/// Base points awarded for participation.
const BASE_POINTS: u64 = 500;

/// Maximum time bonus achievable.
const MAX_TIME_BONUS: u64 = 500;

/// Penalty per hint used (in points).
const HINT_PENALTY_PER_HINT: u64 = 50;

/// Maximum total hint penalty (caps at 500 so base - max_penalty >= 0).
const MAX_HINT_PENALTY: u64 = 500;

/// Calculate a contributor's score for a challenge session.
///
/// # Arguments
/// * `time_delta` - Elapsed time in seconds relative to the target time.
///   - Positive: solver took longer than target (smaller bonus).
///   - Negative: solver finished faster than target (capped at max bonus).
///   - Zero: solver matched the target exactly.
/// * `hints_used` - Number of hints the solver consumed. More hints = lower score.
/// * `session_duration` - Total session wall-clock time in seconds.
///   Used for duration normalization; floored internally to avoid division by zero.
///
/// # Scoring Formula
/// ```text
/// score = clamp(BASE_POINTS + time_bonus - hint_penalty, MIN_SCORE, MAX_SCORE)
/// ```
///
/// # Guarantees
/// - Score is always in `[MIN_SCORE, MAX_SCORE]`.
/// - Hint penalties are monotonic: more hints never increase the score.
/// - Handles negative time deltas, hint-count overflow, and zero-duration sessions.
pub fn calculate_score(time_delta: i64, hints_used: u32, session_duration: u64) -> u64 {
    // Time bonus: negative delta = faster = higher bonus, capped at MAX_TIME_BONUS
    let time_bonus = {
        let raw = if time_delta < 0 {
            MAX_TIME_BONUS
        } else {
            MAX_TIME_BONUS.saturating_sub(time_delta as u64)
        };
        raw.min(MAX_TIME_BONUS)
    };

    // Hint penalty: each hint costs HINT_PENALTY_PER_HINT, capped at MAX_HINT_PENALTY.
    // Use saturating arithmetic to handle u32 -> u64 overflow safely.
    let hint_penalty = {
        let raw = (hints_used as u64).saturating_mul(HINT_PENALTY_PER_HINT);
        raw.min(MAX_HINT_PENALTY)
    };

    // Floor session_duration to 1 to prevent division-by-zero semantics;
    // here it's used for optional future normalization but kept for safety.
    let _effective_duration = session_duration.max(1);

    let raw_score = BASE_POINTS
        .saturating_add(time_bonus)
        .saturating_sub(hint_penalty);

    raw_score.min(MAX_SCORE)
}

/// Calculate score and return it alongside the individual components for inspection.
pub fn calculate_score_detailed(time_delta: i64, hints_used: u32, session_duration: u64) -> ScoreDetail {
    let time_bonus = {
        let raw = if time_delta < 0 {
            MAX_TIME_BONUS
        } else {
            MAX_TIME_BONUS.saturating_sub(time_delta as u64)
        };
        raw.min(MAX_TIME_BONUS)
    };

    let hint_penalty = {
        let raw = (hints_used as u64).saturating_mul(HINT_PENALTY_PER_HINT);
        raw.min(MAX_HINT_PENALTY)
    };

    let _effective_duration = session_duration.max(1);

    let raw_score = BASE_POINTS
        .saturating_add(time_bonus)
        .saturating_sub(hint_penalty);

    let score = raw_score.min(MAX_SCORE);

    ScoreDetail {
        score,
        base_points: BASE_POINTS,
        time_bonus,
        hint_penalty,
        effective_duration: _effective_duration,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreDetail {
    pub score: u64,
    pub base_points: u64,
    pub time_bonus: u64,
    pub hint_penalty: u64,
    pub effective_duration: u64,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod strategy;
