#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, 
    Address, Env, Map, Symbol, Vec,
};

mod test;

// --- Error types ---
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ProgramNotFound = 1,
    WaveAlreadyClosed = 2,
    WaveAlreadySettled = 3,
    WaveNotFound = 4,
    AlreadyInitialized = 5,
    NotInitialized = 6,
    Unauthorized = 7,
    ProgramAlreadyExists = 8,
    WaveAlreadyOpen = 9,
}

// --- Data structures ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveContribution {
    pub wave_id: u64,
    pub points: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveMeta {
    pub wave_id: u64,
    pub program_id: u64,
    pub opened_at: u64,
    pub closed_at: Option<u64>,
    pub total_points: u64,
    pub status: WaveStatus,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WaveStatus {
    Open = 0,
    Closed = 1,
    Settled = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramMeta {
    pub program_id: u64,
    pub name: Symbol,
    pub admin: Address, // creator/admin of the program
}

// --- Storage keys ---
#[contracttype]
pub enum DataKey {
    Admin,                       // Global contract admin
    SettlementContract,          // Authorized settlement contract address
    Programs,                    // Map<u64, ProgramMeta>
    Waves,                       // Map<u64, WaveMeta>
    WaveCounter,                 // u64
    ProgramActiveWave(u64),      // program_id -> wave_id
    Contributions(Address, u64), // (contributor, wave_id) -> WaveContribution
    History(Address),            // contributor -> Vec<u64> (wave IDs)
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the contract with an admin and settlement contract address.
    pub fn initialize(env: Env, admin: Address, settlement_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SettlementContract, &settlement_contract);
    }

    /// Update the authorized settlement contract address. Only callable by admin.
    pub fn set_settlement(env: Env, new_settlement: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        env.storage().instance().set(&DataKey::SettlementContract, &new_settlement);
    }

    /// Register a new program.
    pub fn register_program(env: Env, program_id: u64, name: Symbol, admin: Address) {
        let mut programs: Map<u64, ProgramMeta> = env.storage().instance()
            .get(&DataKey::Programs)
            .unwrap_or(Map::new(&env));

        if programs.contains_key(program_id) {
            panic!("Program already exists");
        }

        programs.set(program_id, ProgramMeta { program_id, name, admin });
        env.storage().instance().set(&DataKey::Programs, &programs);
    }

    /// Open a new wave for a program.
    pub fn open_wave(env: Env, program_id: u64) -> Result<u64, Error> {
        let programs: Map<u64, ProgramMeta> = env.storage().instance()
            .get(&DataKey::Programs)
            .ok_or(Error::NotInitialized)?;
        
        if !programs.contains_key(program_id) {
            return Err(Error::ProgramNotFound);
        }

        // Check if there is already an active wave for this program
        let active_key = DataKey::ProgramActiveWave(program_id);
        if env.storage().instance().has(&active_key) {
            return Err(Error::WaveAlreadyOpen);
        }

        let mut wave_counter: u64 = env.storage().instance().get(&DataKey::WaveCounter).unwrap_or(0);
        wave_counter += 1;
        let wave_id = wave_counter;
        env.storage().instance().set(&DataKey::WaveCounter, &wave_counter);

        let wave = WaveMeta {
            wave_id,
            program_id,
            opened_at: env.ledger().timestamp(),
            closed_at: None,
            total_points: 0,
            status: WaveStatus::Open,
        };

        let mut waves: Map<u64, WaveMeta> = env.storage().instance().get(&DataKey::Waves).unwrap_or(Map::new(&env));
        waves.set(wave_id, wave);
        env.storage().instance().set(&DataKey::Waves, &waves);
        
        // Track as active wave for the program
        env.storage().instance().set(&active_key, &wave_id);

        env.events().publish((Symbol::new(&env, "WaveOpened"), program_id), (wave_id, env.ledger().timestamp()));

        Ok(wave_id)
    }

    /// Close a wave.
    pub fn close_wave(env: Env, wave_id: u64, total_points: u64) -> Result<(), Error> {
        let mut waves: Map<u64, WaveMeta> = env.storage().instance().get(&DataKey::Waves).ok_or(Error::WaveNotFound)?;
        let mut wave = waves.get(wave_id).ok_or(Error::WaveNotFound)?;

        if wave.status != WaveStatus::Open {
            return Err(Error::WaveAlreadyClosed);
        }

        wave.closed_at = Some(env.ledger().timestamp());
        wave.total_points = total_points;
        wave.status = WaveStatus::Closed;

        waves.set(wave_id, wave.clone());
        env.storage().instance().set(&DataKey::Waves, &waves);

        // Clear active wave for the program
        env.storage().instance().remove(&DataKey::ProgramActiveWave(wave.program_id));

        env.events().publish((Symbol::new(&env, "WaveClosed"), wave.program_id), (wave_id, env.ledger().timestamp(), total_points));
        Ok(())
    }

    /// Record a contribution. Only callable by settlement contract.
    pub fn record_contribution(env: Env, wave_id: u64, address: Address, points: u32) {
        let settlement: Address = env.storage().instance().get(&DataKey::SettlementContract).expect("Settlement not set");
        settlement.require_auth();

        let contribution = WaveContribution {
            wave_id,
            points,
            timestamp: env.ledger().timestamp(),
        };

        let key = DataKey::Contributions(address.clone(), wave_id);
        env.storage().persistent().set(&key, &contribution);

        let history_key = DataKey::History(address.clone());
        let mut history: Vec<u64> = env.storage().persistent().get(&history_key).unwrap_or(Vec::new(&env));
        if !history.contains(wave_id) {
            history.push_back(wave_id);
            env.storage().persistent().set(&history_key, &history);
        }
    }

    /// Get contribution history for an address.
    pub fn contributor_record(env: Env, address: Address) -> Vec<WaveContribution> {
        let history_key = DataKey::History(address.clone());
        let wave_ids: Vec<u64> = env.storage().persistent().get(&history_key).unwrap_or(Vec::new(&env));
        
        let mut contributions = Vec::new(&env);
        for wave_id in wave_ids.iter() {
            let key = DataKey::Contributions(address.clone(), wave_id);
            if let Some(c) = env.storage().persistent().get::<_, WaveContribution>(&key) {
                contributions.push_back(c);
            }
        }
        contributions
    }

    pub fn get_program(env: Env, program_id: u64) -> Option<ProgramMeta> {
        let programs: Map<u64, ProgramMeta> = env.storage().instance().get(&DataKey::Programs)?;
        programs.get(program_id)
    }

    pub fn get_wave(env: Env, wave_id: u64) -> Option<WaveMeta> {
        let waves: Map<u64, WaveMeta> = env.storage().instance().get(&DataKey::Waves)?;
        waves.get(wave_id)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("Not initialized")
    }

    pub fn get_settlement(env: Env) -> Address {
        env.storage().instance().get(&DataKey::SettlementContract).expect("Not initialized")
    }
}
