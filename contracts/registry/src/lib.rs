#![no_std]

use soroban_sdk::{contract, contractimpl, Symbol, Env, Address};

#[cfg(test)]
mod tests;

/// Event emitted when a new program is registered
/// Fields: program_id (Symbol), name (Symbol), organizer (Address)
#[derive(Clone)]
pub struct ProgramRegisteredEvent {
    pub program_id: Symbol,
    pub name: Symbol,
    pub organizer: Address,
}

/// Event emitted when a wave is opened for a program
/// Fields: wave_id (Symbol), program_id (Symbol)
#[derive(Clone)]
pub struct WaveOpenedEvent {
    pub wave_id: Symbol,
    pub program_id: Symbol,
}

/// Event emitted when a wave is closed
/// Fields: wave_id (Symbol), total_points (i128)
#[derive(Clone)]
pub struct WaveClosedEvent {
    pub wave_id: Symbol,
    pub total_points: i128,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Register a new program in the registry
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - Unique identifier for the program
    /// * `name` - Name of the program
    /// * `organizer` - Address of the program organizer
    ///
    /// # Emits
    /// * `registry_program_registered` event with program_id, name, and organizer
    pub fn register_program(env: Env, program_id: Symbol, name: Symbol, organizer: Address) {
        // Require authorization from organizer
        organizer.require_auth();

        // Emit program registered event
        env.events().publish(
            (Symbol::new(&env, "registry_program_registered"),),
            (program_id, name, organizer),
        );
    }

    /// Open a new wave for a program
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `wave_id` - Unique identifier for the wave
    /// * `program_id` - The program this wave belongs to
    ///
    /// # Emits
    /// * `registry_wave_opened` event with wave_id and program_id
    pub fn open_wave(env: Env, wave_id: Symbol, program_id: Symbol) {
        // Validate wave_id and program_id are not empty
        if wave_id.len() == 0 || program_id.len() == 0 {
            panic!("Wave ID and Program ID must not be empty");
        }

        // Emit wave opened event
        env.events().publish(
            (Symbol::new(&env, "registry_wave_opened"),),
            (wave_id, program_id),
        );
    }

    /// Close a wave and finalize its state
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `wave_id` - Identifier of the wave to close
    /// * `total_points` - Total points distributed in this wave
    ///
    /// # Emits
    /// * `registry_wave_closed` event with wave_id and total_points
    pub fn close_wave(env: Env, wave_id: Symbol, total_points: i128) {
        // Validate total_points
        if total_points < 0 {
            panic!("Total points cannot be negative");
        }

        // Emit wave closed event
        env.events().publish(
            (Symbol::new(&env, "registry_wave_closed"),),
            (wave_id, total_points),
        );
    }

    /// Update program information
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `program_id` - Identifier of the program
    /// * `name` - Updated program name
    /// * `organizer` - Organizer address
    ///
    /// # Emits
    /// * `registry_program_registered` event (re-registers the program)
    pub fn update_program(env: Env, program_id: Symbol, name: Symbol, organizer: Address) {
        // Require authorization from organizer
        organizer.require_auth();

        // Emit program registered event (acts as update)
        env.events().publish(
            (Symbol::new(&env, "registry_program_registered"),),
            (program_id, name, organizer),
        );
    }
}
