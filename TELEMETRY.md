# Structured Logging and Session Telemetry

This document describes the structured logging and session telemetry system implemented across all mesh-contract smart contracts. The telemetry system is **opt-in** and disabled by default to preserve privacy and minimize gas costs.

## Overview

The telemetry system provides:
- **Structured Logging**: Capture contract events with timestamps, function names, caller addresses, and context
- **Session Metrics**: Track usage patterns per address (call counts, success/failure rates)
- **Opt-in Privacy**: Telemetry is disabled by default and must be explicitly enabled
- **Circular Buffer**: Efficient log storage with automatic rotation (default: 100 entries)

## Architecture

### Core Components

#### 1. TelemetryStatus
```rust
pub enum TelemetryStatus {
    Disabled = 0,  // Default - no telemetry data collected
    Enabled = 1,   // Active - telemetry data is being collected
}
```

#### 2. LogEntry
Structured log entry containing:
- `timestamp`: Ledger timestamp when the event occurred
- `function_name`: Name of the contract function called
- `caller`: Address that initiated the call
- `context`: Additional context data (JSON-like string)
- `level`: Log level (0=Debug, 1=Info, 2=Warn, 3=Error)

#### 3. SessionMetrics
Per-address session tracking:
- `session_start`: When the session began
- `last_activity`: Most recent activity timestamp
- `total_calls`: Total number of function calls
- `successful_calls`: Number of successful calls
- `failed_calls`: Number of failed calls
- `total_gas_used`: Accumulated gas consumption

## Implementation

### Telemetry Manager

Each contract has a `TelemetryManager` module providing:

```rust
// Check if telemetry is enabled
pub fn is_enabled(env: &Env) -> bool

// Enable/disable telemetry (admin only)
pub fn enable(env: &Env)
pub fn disable(env: &Env)

// Logging functions
pub fn log(env: &Env, function_name: &str, caller: &str, context: &str, level: u32)
pub fn debug(env: &Env, function_name: &str, caller: &str, context: &str)
pub fn info(env: &Env, function_name: &str, caller: &str, context: &str)
pub fn warn(env: &Env, function_name: &str, caller: &str, context: &str)
pub fn error(env: &Env, function_name: &str, caller: &str, context: &str)

// Session tracking
pub fn start_session(env: &Env, caller: Address)
pub fn record_success(env: &Env, caller: Address)
pub fn record_failure(env: &Env, caller: Address)
pub fn get_session_metrics(env: &Env, caller: Address) -> Option<SessionMetrics>

// Log management
pub fn get_logs(env: &Env, count: u32) -> Vec<LogEntry>
pub fn clear_logs(env: &Env)
```

### Contract Integration

Telemetry is integrated into all three contracts:

#### Registry Contract
- `initialize()` - Logs contract initialization
- `set_onboarder()` - Tracks onboarder configuration
- `register_program()` - Logs program registration
- `open_wave()` - Tracks wave openings
- `close_wave()` - Tracks wave closures
- `record_contribution()` - Logs contribution recording
- `set_settlement()` - Tracks settlement contract updates
- `get_contributor_performance()` - Logs performance queries
- `get_program_difficulty()` - Logs difficulty queries
- `set_program_difficulty()` - Tracks difficulty changes

#### Escrow Contract
- `initialize()` - Logs contract initialization
- `open_wave()` - Tracks escrow creation
- `fund_wave()` - Logs funding events
- `fund()` - Tracks program funding
- `get_program_balance()` - Logs balance queries
- `release()` - Tracks fund releases

#### Settlement Contract
- `init()` - Logs contract initialization
- `settle()` - Tracks wave settlements
- `is_wave_settled()` - Logs settlement queries
- `get_registry_contract()` - Logs registry queries
- `get_settlement_count()` - Logs count queries

## Usage

### Enabling Telemetry

Telemetry is disabled by default. To enable it, the admin must call the enable function:

```rust
// In registry contract
TelemetryManager::enable(&env);

// In escrow contract
TelemetryManager::enable(&env);

// In settlement contract
TelemetryManager::enable(&env);
```

### Disabling Telemetry

```rust
// Disable telemetry at any time
TelemetryManager::disable(&env);
```

### Querying Logs

```rust
// Get the last 10 log entries
let logs = TelemetryManager::get_logs(&env, 10);

// Clear all logs
TelemetryManager::clear_logs(&env);
```

### Querying Session Metrics

```rust
// Get session metrics for a specific address
let metrics = TelemetryManager::get_session_metrics(&env, address);
```

## Privacy & Gas Considerations

### Privacy
- **Opt-in by design**: Telemetry is disabled by default
- **No PII collection**: Only contract addresses and function names are logged
- **Context is generic**: Logged context contains only operational data
- **User-controlled**: Admins can disable telemetry at any time

### Gas Costs
When telemetry is **disabled**:
- All telemetry functions return immediately
- No storage writes occur
- No events are published
- **Zero gas overhead**

When telemetry is **enabled**:
- Each logged function call adds ~100-200 gas
- Session tracking adds minimal overhead
- Logs are stored in a circular buffer to limit storage costs
- Default buffer size: 100 entries (configurable via `MaxLogs`)

## Storage Keys

Telemetry data is stored using these keys:

### Registry & Settlement Contracts
```rust
TelemetryDataKey::Status                    // Telemetry enabled/disabled
TelemetryDataKey::SessionMetrics(Address)   // Per-address session data
TelemetryDataKey::Logs(u32)                 // Circular buffer of log entries
TelemetryDataKey::LogCounter                // Current log index
TelemetryDataKey::MaxLogs                   // Maximum logs to retain
```

### Escrow Contract
Uses the same structure but with string-based keys for compatibility.

## Events

When telemetry is enabled, the following events are published for off-chain indexing:

```
log(function_name) -> (timestamp, level, context)
```

These events allow external monitoring systems to track contract activity without reading on-chain storage.

## Testing

Each contract includes comprehensive tests for the telemetry module:

```rust
#[test]
fn test_telemetry_enable_disable() { ... }

#[test]
fn test_logging_when_disabled() { ... }

#[test]
fn test_logging_when_enabled() { ... }

#[test]
fn test_session_metrics() { ... }
```

Run tests with:
```bash
cargo test --package registry
cargo test --package escrow
cargo test --package settlement
```

## Configuration

### Default Settings
- Telemetry status: **Disabled**
- Max logs: **100**
- Log levels: 0-3 (Debug, Info, Warn, Error)

### Customization

To modify the maximum number of logs retained:

```rust
// Set custom max logs (must be done before enabling telemetry)
env.storage().instance().set(&TelemetryDataKey::MaxLogs, &200u32);
```

## Best Practices

1. **Enable only when needed**: Turn on telemetry for debugging or monitoring periods
2. **Regular cleanup**: Call `clear_logs()` periodically to manage storage costs
3. **Monitor gas usage**: Track the gas overhead when telemetry is enabled
4. **Privacy first**: Always inform users if telemetry is enabled on shared contracts

## Migration Guide

### For Existing Contracts

To add telemetry to an existing contract:

1. Create `src/telemetry.rs` with the TelemetryManager implementation
2. Add `mod telemetry;` and `use telemetry::TelemetryManager;` to `lib.rs`
3. Add telemetry calls to each function you want to track
4. Initialize telemetry in the `initialize`/`init` function
5. Test thoroughly to ensure no breaking changes

### For New Contracts

Follow the pattern established in the existing contracts:
- Registry: `contracts/registry/src/telemetry.rs`
- Escrow: `contracts/escrow/src/telemetry.rs`
- Settlement: `contracts/settlement/src/telemetry.rs`

## Troubleshooting

### Telemetry not working?
- Verify telemetry is enabled: `TelemetryManager::is_enabled(&env)`
- Check that the admin address is correct
- Ensure you're not hitting storage limits

### High gas costs?
- Reduce `MaxLogs` value
- Disable telemetry when not needed
- Use `clear_logs()` to reset the circular buffer

### Logs not appearing?
- Verify telemetry is enabled
- Check that the function is actually being called
- Ensure the log level is appropriate (debug logs may be filtered)

## Future Enhancements

Potential improvements for future versions:
- Configurable log levels per function
- Aggregated metrics (hourly/daily summaries)
- Off-chain indexing integration
- Gas usage tracking per function
- Anomaly detection for unusual patterns