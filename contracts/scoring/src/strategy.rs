use proptest::prelude::*;

/// Strategy for generating arbitrary time_delta values.
/// Covers negative, zero, and positive values including extreme ranges.
pub fn time_delta_strategy() -> impl Strategy<Value = i64> {
    prop_oneof![
        // Negative values (fast solver)
        (-1_000_000i64..-1),
        // Zero (exact target)
        Just(0i64),
        // Positive values (slow solver)
        (1i64..1_000_000),
        // Edge cases
        Just(i64::MIN),
        Just(i64::MAX),
        Just(-1i64),
        Just(1i64),
    ]
}

/// Strategy for generating arbitrary hints_used values.
/// Covers zero, typical ranges, and u32::MAX for overflow testing.
pub fn hints_used_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        // Zero hints
        Just(0u32),
        // Typical range (0..=20)
        (0u32..=20),
        // Large values approaching overflow
        (1000u32..=u32::MAX),
        // Specific edge values
        Just(u32::MAX),
        Just(1u32),
        Just(10u32),
    ]
}

/// Strategy for generating arbitrary session_duration values.
/// Covers zero, typical, and extreme durations.
pub fn session_duration_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        // Zero duration (edge case)
        Just(0u64),
        // Typical ranges
        (1u64..=86400),
        // Very long sessions
        (86401u64..=31_536_000),
        // Extreme values
        Just(u64::MAX),
        Just(1u64),
    ]
}

/// Strategy for generating a complete set of scoring inputs.
#[allow(dead_code)]
pub fn scoring_inputs_strategy() -> impl Strategy<Value = (i64, u32, u64)> {
    (
        time_delta_strategy(),
        hints_used_strategy(),
        session_duration_strategy(),
    )
}
