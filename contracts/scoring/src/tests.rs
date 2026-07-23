use proptest::prelude::*;

use crate::{calculate_score, calculate_score_detailed, MAX_SCORE, MIN_SCORE};
use crate::strategy::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property 1: Score is never negative.
    /// For ALL valid inputs, the score must be >= MIN_SCORE (0).
    #[test]
    fn score_is_never_negative(
        time_delta in time_delta_strategy(),
        hints_used in hints_used_strategy(),
        session_duration in session_duration_strategy(),
    ) {
        let score = calculate_score(time_delta, hints_used, session_duration);
        prop_assert!(
            score >= MIN_SCORE,
            "Score {} is negative for inputs (time_delta={}, hints_used={}, session_duration={})",
            score, time_delta, hints_used, session_duration,
        );
    }

    /// Property 2: Score never exceeds the documented maximum.
    /// For ALL valid inputs, the score must be <= MAX_SCORE (1000).
    #[test]
    fn score_never_exceeds_max(
        time_delta in time_delta_strategy(),
        hints_used in hints_used_strategy(),
        session_duration in session_duration_strategy(),
    ) {
        let score = calculate_score(time_delta, hints_used, session_duration);
        prop_assert!(
            score <= MAX_SCORE,
            "Score {} exceeds MAX_SCORE {} for inputs (time_delta={}, hints_used={}, session_duration={})",
            score, MAX_SCORE, time_delta, hints_used, session_duration,
        );
    }

    /// Property 3: Hint penalties are monotonic.
    /// For the same time_delta and session_duration, increasing hints_used
    /// must never increase the score.
    #[test]
    fn hint_penalties_are_monotonic(
        time_delta in time_delta_strategy(),
        session_duration in session_duration_strategy(),
        fewer_hints in 0u32..=50,
        more_hints in 51u32..=100,
    ) {
        let score_fewer = calculate_score(time_delta, fewer_hints, session_duration);
        let score_more = calculate_score(time_delta, more_hints, session_duration);
        prop_assert!(
            score_more <= score_fewer,
            "Score with more hints ({}) > score with fewer hints ({}) \
             for hints_used {} vs {}, time_delta={}, session_duration={}",
            score_more, score_fewer, more_hints, fewer_hints, time_delta, session_duration,
        );
    }

    /// Property 4: Negative time_delta never reduces score below zero-delta score.
    /// Finishing faster (negative delta) should yield a score >= finishing at target (delta=0).
    #[test]
    fn negative_time_delta_not_worse_than_zero(
        hints_used in 0u32..=20,
        session_duration in session_duration_strategy(),
    ) {
        let score_zero_delta = calculate_score(0, hints_used, session_duration);
        let score_negative_delta = calculate_score(-1000, hints_used, session_duration);
        prop_assert!(
            score_negative_delta >= score_zero_delta,
            "Negative time delta score ({}) < zero delta score ({})",
            score_negative_delta, score_zero_delta,
        );
    }

    /// Property 5: Zero-duration sessions do not cause panics or undefined behavior.
    /// The function must return a valid score for session_duration = 0.
    #[test]
    fn zero_duration_session_produces_valid_score(
        time_delta in time_delta_strategy(),
        hints_used in hints_used_strategy(),
    ) {
        let score = calculate_score(time_delta, hints_used, 0);
        prop_assert!(
            score >= MIN_SCORE && score <= MAX_SCORE,
            "Zero-duration score {} out of range [{}, {}]",
            score, MIN_SCORE, MAX_SCORE,
        );
    }

    /// Property 6: Hint overflow (u32::MAX) does not cause panic or invalid score.
    /// Using the maximum u32 value for hints must be handled safely.
    #[test]
    fn hint_overflow_produces_valid_score(
        time_delta in time_delta_strategy(),
        session_duration in session_duration_strategy(),
    ) {
        let score = calculate_score(time_delta, u32::MAX, session_duration);
        prop_assert!(
            score >= MIN_SCORE && score <= MAX_SCORE,
            "Hint overflow score {} out of range [{}, {}]",
            score, MIN_SCORE, MAX_SCORE,
        );
    }

    /// Property 7: Detailed score components are internally consistent.
    /// The returned score equals base + time_bonus - hint_penalty, clamped to range.
    #[test]
    fn detailed_score_components_are_consistent(
        time_delta in time_delta_strategy(),
        hints_used in hints_used_strategy(),
        session_duration in session_duration_strategy(),
    ) {
        let detail = calculate_score_detailed(time_delta, hints_used, session_duration);
        let expected = (detail.base_points as i64)
            + (detail.time_bonus as i64)
            - (detail.hint_penalty as i64);
        let clamped = expected.clamp(MIN_SCORE as i64, MAX_SCORE as i64) as u64;
        prop_assert_eq!(
            detail.score, clamped,
            "Score {} does not match clamped computation {} \
             (base={}, time_bonus={}, hint_penalty={})",
            detail.score, clamped, detail.base_points, detail.time_bonus, detail.hint_penalty,
        );
    }

    /// Property 8: Effective duration is always >= 1 (no division by zero).
    #[test]
    fn effective_duration_always_positive(
        time_delta in time_delta_strategy(),
        hints_used in hints_used_strategy(),
        session_duration in session_duration_strategy(),
    ) {
        let detail = calculate_score_detailed(time_delta, hints_used, session_duration);
        prop_assert!(
            detail.effective_duration >= 1,
            "effective_duration {} is less than 1",
            detail.effective_duration,
        );
    }
}
