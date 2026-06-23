use soroban_sdk::{Address, Map};

/// Program vault data structure
/// Stores token address and balance for each program
pub struct ProgramVault {
    pub token: Address,
    pub balance: i128,
}

/// Storage key for program balances
pub type ProgramBalances = Map<u64, (Address, i128)>;
