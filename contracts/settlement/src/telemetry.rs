#![no_std]
use soroban_sdk::{contracttype, Address, Env, String, Vec};

/// Telemetry configuration status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryStatus {
    Disabled = 0,
    Enabled = 1,
}

/// Structured log entry for contract events
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub function_name: String,
    pub caller: Address,
    pub context: String,
    pub level: u32,
}

/// Session metrics for tracking contract usage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMetrics {
    pub session_start: u64,
    pub last_activity: u64,
    pub total_calls: u32,
    pub successful_calls: u32,
    pub failed_calls: u32,
    pub total_gas_used: u64,
}

/// Storage keys for telemetry data
#[contracttype]
pub enum TelemetryDataKey {
    Status,
    SessionMetrics(Address),
    Logs(u32),
    LogCounter,
    MaxLogs,
}

/// Telemetry manager for structured logging and session tracking
pub struct TelemetryManager;

impl TelemetryManager {
    /// Check if telemetry is enabled
    pub fn is_enabled(env: &Env) -> bool {
        let key = TelemetryDataKey::Status;
        if let Some(status) = env.storage().instance().get::<_, TelemetryStatus>(&key) {
            status == TelemetryStatus::Enabled
        } else {
            false
        }
    }

    /// Enable telemetry (admin only)
    pub fn enable(env: &Env) {
        env.storage().instance().set(&TelemetryDataKey::Status, &TelemetryStatus::Enabled);
        Self::log(env, "Telemetry", "system", "Telemetry enabled", 1);
    }

    /// Disable telemetry (admin only)
    pub fn disable(env: &Env) {
        env.storage().instance().set(&TelemetryDataKey::Status, &TelemetryStatus::Disabled);
    }

    /// Log a structured message
    pub fn log(env: &Env, function_name: &str, caller: &str, context: &str, level: u32) {
        if !Self::is_enabled(env) {
            return;
        }

        let timestamp = env.ledger().timestamp();
        let log_entry = LogEntry {
            timestamp,
            function_name: String::from_str(env, function_name),
            caller: Address::from_string(&String::from_str(env, caller)),
            context: String::from_str(env, context),
            level,
        };

        let mut counter: u32 = env
            .storage()
            .instance()
            .get(&TelemetryDataKey::LogCounter)
            .unwrap_or(0);
        counter += 1;

        let max_logs: u32 = env
            .storage()
            .instance()
            .get(&TelemetryDataKey::MaxLogs)
            .unwrap_or(100);

        let log_index = (counter - 1) % max_logs;
        env.storage()
            .instance()
            .set(&TelemetryDataKey::Logs(log_index), &log_entry);
        env.storage()
            .instance()
            .set(&TelemetryDataKey::LogCounter, &counter);

        env.events().publish(
            (soroban_sdk::symbol_short!("log"), function_name),
            (timestamp, level, context),
        );
    }

    /// Log debug message
    pub fn debug(env: &Env, function_name: &str, caller: &str, context: &str) {
        Self::log(env, function_name, caller, context, 0);
    }

    /// Log info message
    pub fn info(env: &Env, function_name: &str, caller: &str, context: &str) {
        Self::log(env, function_name, caller, context, 1);
    }

    /// Log warning message
    pub fn warn(env: &Env, function_name: &str, caller: &str, context: &str) {
        Self::log(env, function_name, caller, context, 2);
    }

    /// Log error message
    pub fn error(env: &Env, function_name: &str, caller: &str, context: &str) {
        Self::log(env, function_name, caller, context, 3);
    }

    /// Start or update a session for a caller
    pub fn start_session(env: &Env, caller: Address) {
        if !Self::is_enabled(env) {
            return;
        }

        let timestamp = env.ledger().timestamp();
        let session = SessionMetrics {
            session_start: timestamp,
            last_activity: timestamp,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            total_gas_used: 0,
        };

        env.storage()
            .persistent()
            .set(&TelemetryDataKey::SessionMetrics(caller), &session);
    }

    /// Record a successful function call in the session
    pub fn record_success(env: &Env, caller: Address) {
        if !Self::is_enabled(env) {
            return;
        }

        let mut session: SessionMetrics = env
            .storage()
            .persistent()
            .get(&TelemetryDataKey::SessionMetrics(caller.clone()))
            .unwrap_or_else(|| SessionMetrics {
                session_start: env.ledger().timestamp(),
                last_activity: env.ledger().timestamp(),
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_gas_used: 0,
            });

        session.total_calls += 1;
        session.successful_calls += 1;
        session.last_activity = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&TelemetryDataKey::SessionMetrics(caller), &session);
    }

    /// Record a failed function call in the session
    pub fn record_failure(env: &Env, caller: Address) {
        if !Self::is_enabled(env) {
            return;
        }

        let mut session: SessionMetrics = env
            .storage()
            .persistent()
            .get(&TelemetryDataKey::SessionMetrics(caller.clone()))
            .unwrap_or_else(|| SessionMetrics {
                session_start: env.ledger().timestamp(),
                last_activity: env.ledger().timestamp(),
                total_calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_gas_used: 0,
            });

        session.total_calls += 1;
        session.failed_calls += 1;
        session.last_activity = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&TelemetryDataKey::SessionMetrics(caller), &session);
    }

    /// Get session metrics for a caller
    pub fn get_session_metrics(env: &Env, caller: Address) -> Option<SessionMetrics> {
        env.storage()
            .persistent()
            .get(&TelemetryDataKey::SessionMetrics(caller))
    }

    /// Get recent log entries
    pub fn get_logs(env: &Env, count: u32) -> Vec<LogEntry> {
        if !Self::is_enabled(env) {
            return Vec::new(env);
        }

        let max_logs: u32 = env
            .storage()
            .instance()
            .get(&TelemetryDataKey::MaxLogs)
            .unwrap_or(100);

        let log_counter: u32 = env
            .storage()
            .instance()
            .get(&TelemetryDataKey::LogCounter)
            .unwrap_or(0);

        let actual_count = count.min(max_logs).min(log_counter);
        let mut logs = Vec::new(env);

        if log_counter == 0 || actual_count == 0 {
            return logs;
        }

        let start_index = if log_counter >= max_logs {
            log_counter % max_logs
        } else {
            0
        };

        for i in 0..actual_count {
            let index = (start_index + log_counter - actual_count + i) % max_logs;
            if let Some(log_entry) = env
                .storage()
                .instance()
                .get::<_, LogEntry>(&TelemetryDataKey::Logs(index))
            {
                logs.push_back(log_entry);
            }
        }

        logs
    }

    /// Clear old logs (maintenance function)
    pub fn clear_logs(env: &Env) {
        if !Self::is_enabled(env) {
            return;
        }

        env.storage()
            .instance()
            .set(&TelemetryDataKey::LogCounter, &0u32);
        Self::info(env, "Telemetry", "system", "Logs cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_telemetry_enable_disable() {
        let env = Env::default();
        
        assert!(!TelemetryManager::is_enabled(&env));
        
        TelemetryManager::enable(&env);
        assert!(TelemetryManager::is_enabled(&env));
        
        TelemetryManager::disable(&env);
        assert!(!TelemetryManager::is_enabled(&env));
    }

    #[test]
    fn test_logging_when_disabled() {
        let env = Env::default();
        let caller = Address::generate(&env);
        
        TelemetryManager::log(&env, "test_func", &caller.to_string(), "test context", 1);
        TelemetryManager::info(&env, "test_func", &caller.to_string(), "test info");
    }

    #[test]
    fn test_logging_when_enabled() {
        let env = Env::default();
        let caller = Address::generate(&env);
        
        TelemetryManager::enable(&env);
        
        TelemetryManager::info(&env, "test_func", &caller.to_string(), "test info");
        
        let logs = TelemetryManager::get_logs(&env, 10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_session_metrics() {
        let env = Env::default();
        let caller = Address::generate(&env);
        
        TelemetryManager::enable(&env);
        
        assert!(TelemetryManager::get_session_metrics(&env, caller.clone()).is_none());
        
        TelemetryManager::start_session(&env, caller.clone());
        let session = TelemetryManager::get_session_metrics(&env, caller.clone()).unwrap();
        assert_eq!(session.total_calls, 0);
        
        TelemetryManager::record_success(&env, caller.clone());
        let session = TelemetryManager::get_session_metrics(&env, caller.clone()).unwrap();
        assert_eq!(session.total_calls, 1);
        assert_eq!(session.successful_calls, 1);
        
        TelemetryManager::record_failure(&env, caller.clone());
        let session = TelemetryManager::get_session_metrics(&env, caller.clone()).unwrap();
        assert_eq!(session.total_calls, 2);
        assert_eq!(session.failed_calls, 1);
    }
}