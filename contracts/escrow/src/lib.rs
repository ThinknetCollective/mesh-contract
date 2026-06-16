#![no_std]

use soroban_sdk::{contract, contractimpl, Symbol, Env, Address};

#[cfg(test)]
mod tests;

/// Event emitted when funds are deposited into escrow
/// Fields: program_id (Symbol), amount (i128)
#[derive(Clone)]
pub struct EscrowFundedEvent {
    pub program_id: Symbol,
    pub amount: i128,
}

/// Event emitted when escrow funds are released
/// Fields: wave_id (Symbol), recipient (Address), amount (i128)
#[derive(Clone)]
pub struct EscrowReleasedEvent {
    pub wave_id: Symbol,
    pub recipient: Address,
    pub amount: i128,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Fund escrow with specified amount for a program
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - Identifier for the program receiving funds
    /// * `amount` - Amount to deposit in escrow
    ///
    /// # Emits
    /// * `escrow_funded` event with program_id and amount
    pub fn fund(env: Env, program_id: Symbol, amount: i128) {
        // Validate amount
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Emit escrow_funded event
        env.events().publish(
            (Symbol::new(&env, "escrow_funded"),),
            (program_id, amount),
        );
    }

    /// Release escrow funds to a recipient for a wave
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `wave_id` - Identifier for the wave
    /// * `recipient` - Address of the fund recipient
    /// * `amount` - Amount to release from escrow
    ///
    /// # Emits
    /// * `escrow_released` event with wave_id, recipient, and amount
    pub fn release(env: Env, wave_id: Symbol, recipient: Address, amount: i128) {
        // Validate amount
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Validate recipient
        recipient.require_auth();

        // Emit escrow_released event
        env.events().publish(
            (Symbol::new(&env, "escrow_released"),),
            (wave_id, recipient, amount),
        );
    }
}
