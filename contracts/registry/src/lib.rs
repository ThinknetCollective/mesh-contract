#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveContribution {
    pub wave_id: u32,
    pub points: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaveStatus {
    Open,
    Closed,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveMeta {
    pub program_id: String,
    pub wave_id: u32,
    pub opened_at: u64,
    pub closed_at: u64,
    pub total_points: u32,
    pub status: WaveStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramMeta {
    pub creator: Address,
    pub metadata: String,
    pub funding_target: u128,
    pub is_active: bool,
}

/// Storage keys for the registry contract state.
#[contracttype]
pub enum DataKey {
    Admin,
    SettlementContract,
    Programs(String),
    Waves(u32),
    WaveCounter,
    Contributions(Address, u32), // contributor, wave_id -> contribution
    History(Address),            // contributor -> Vec<wave_id>
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the contract with an admin and the authorized settlement contract address.
    pub fn initialize(env: Env, admin: Address, settlement_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SettlementContract, &settlement_contract);
        env.storage().instance().set(&DataKey::WaveCounter, &0u32);
    }

    /// Register a new Wave Program. Only callable by admin.
    pub fn register_program(
        env: Env,
        program_id: String,
        creator: Address,
        metadata: String,
        funding_target: u128,
    ) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Programs(program_id.clone())) {
            panic!("program already exists");
        }

        let program = ProgramMeta {
            creator,
            metadata,
            funding_target,
            is_active: true,
        };

        env.storage().persistent().set(&DataKey::Programs(program_id), &program);
    }

    /// Opens a new wave cycle for a program. Returns wave_id.
    pub fn open_wave(env: Env, program_id: String) -> u32 {
        if !env.storage().persistent().has(&DataKey::Programs(program_id.clone())) {
            panic!("program doesn't exist");
        }

        // Increment global wave ID
        let mut counter: u32 = env.storage().instance().get(&DataKey::WaveCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::WaveCounter, &counter);

        let wave_id = counter;
        let wave = WaveMeta {
            program_id: program_id.clone(),
            wave_id,
            opened_at: env.ledger().timestamp(),
            closed_at: 0,
            total_points: 0,
            status: WaveStatus::Open,
        };

        env.storage().persistent().set(&DataKey::Waves(wave_id), &wave);

        // Emit WaveOpened event
        env.events().publish(
            (symbol_short!("wave_open"), program_id, wave_id),
            env.ledger().timestamp(),
        );

        wave_id
    }

    /// Closes an open wave cycle and marks it ready for settlement.
    pub fn close_wave(env: Env, wave_id: u32, total_points: u32) {
        let mut wave: WaveMeta = env
            .storage()
            .persistent()
            .get(&DataKey::Waves(wave_id))
            .expect("wave not found");

        if wave.status != WaveStatus::Open {
            panic!("wave already closed or settled");
        }

        wave.closed_at = env.ledger().timestamp();
        wave.total_points = total_points;
        wave.status = WaveStatus::Closed;

        env.storage().persistent().set(&DataKey::Waves(wave_id), &wave);

        // Emit WaveClosed event
        env.events().publish(
            (symbol_short!("wave_cls"), wave_id, total_points),
            env.ledger().timestamp(),
        );
    }

    /// Record a contribution points entry. Only callable by settlement contract.
    pub fn record_contribution(env: Env, wave_id: u32, address: Address, points: u32) {
        let settlement: Address = env
            .storage().instance()
            .get(&DataKey::SettlementContract)
            .expect("settlement not set");
        settlement.require_auth();

        let wave: WaveMeta = env
            .storage()
            .persistent()
            .get(&DataKey::Waves(wave_id))
            .expect("wave not found");
        
        if wave.status != WaveStatus::Open {
            panic!("wave is not open");
        }

        let contribution = WaveContribution {
            wave_id,
            points,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Contributions(address.clone(), wave_id), &contribution);

        let mut history: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::History(address.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        if !history.contains(wave_id) {
            history.push_back(wave_id);
            env.storage().persistent().set(&DataKey::History(address), &history);
        }
    }

    /// Returns the full contribution history for a contributor.
    pub fn contributor_record(env: Env, address: Address) -> Vec<WaveContribution> {
        let history: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::History(address.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        let mut contributions = Vec::new(&env);
        for wave_id in history.iter() {
            if let Some(contribution) = env.storage().persistent().get::<_, WaveContribution>(&DataKey::Contributions(address.clone(), wave_id)) {
                contributions.push_back(contribution);
            }
        }
        contributions
    }

    pub fn get_wave(env: Env, wave_id: u32) -> Option<WaveMeta> {
        env.storage().persistent().get(&DataKey::Waves(wave_id))
    }

    pub fn get_program(env: Env, program_id: String) -> Option<ProgramMeta> {
        env.storage().persistent().get(&DataKey::Programs(program_id))
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("not initialized")
    }

    pub fn get_settlement(env: Env) -> Address {
        env.storage().instance().get(&DataKey::SettlementContract).expect("not initialized")
    }
    
    pub fn set_settlement(env: Env, new_settlement: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        admin.require_auth();
        env.storage().instance().set(&DataKey::SettlementContract, &new_settlement);
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events, Ledger},
        Address, Env, String,
    };

    fn setup(env: &Env) -> (RegistryContractClient<'static>, Address, Address) {
        let contract_id = env.register_contract(None, RegistryContract);
        let client = RegistryContractClient::new(env, &contract_id);
        
        let admin = Address::generate(env);
        let settlement = Address::generate(env);
        
        client.initialize(&admin, &settlement);
        (client, admin, settlement)
    }

    #[test]
    fn test_initialize_and_getters() {
        let env = Env::default();
        let (client, admin, settlement) = setup(&env);
        
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_settlement(), settlement);
    }

    #[test]
    fn test_wave_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let program_id = String::from_str(&env, "prog1");
        client.register_program(&program_id, &Address::generate(&env), &String::from_str(&env, "meta"), &1000);
        
        // Open Wave
        let timestamp = 123456789;
        env.ledger().with_mut(|li| li.timestamp = timestamp);
        let wave_id = client.open_wave(&program_id);
        assert_eq!(wave_id, 1);
        
        let wave = client.get_wave(&wave_id).unwrap();
        assert_eq!(wave.program_id, program_id);
        assert_eq!(wave.status, WaveStatus::Open);
        assert_eq!(wave.opened_at, timestamp);
        
        // Close Wave
        let close_timestamp = 123457000;
        env.ledger().with_mut(|li| li.timestamp = close_timestamp);
        client.close_wave(&wave_id, &500);
        
        let wave = client.get_wave(&wave_id).unwrap();
        assert_eq!(wave.status, WaveStatus::Closed);
        assert_eq!(wave.closed_at, close_timestamp);
        assert_eq!(wave.total_points, 500);
        
        // Verify events
        let events = env.events().all();
        assert!(events.len() >= 2);
    }

    #[test]
    #[should_panic(expected = "program doesn't exist")]
    fn test_open_wave_non_existent_program() {
        let env = Env::default();
        let (client, _, _) = setup(&env);
        client.open_wave(&String::from_str(&env, "ghost"));
    }

    #[test]
    #[should_panic(expected = "wave already closed or settled")]
    fn test_close_already_closed_wave() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let program_id = String::from_str(&env, "prog1");
        client.register_program(&program_id, &Address::generate(&env), &String::from_str(&env, "meta"), &1000);
        let wave_id = client.open_wave(&program_id);
        
        client.close_wave(&wave_id, &100);
        client.close_wave(&wave_id, &200); // Should panic
    }

    #[test]
    fn test_record_contribution() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let program_id = String::from_str(&env, "prog1");
        client.register_program(&program_id, &Address::generate(&env), &String::from_str(&env, "meta"), &1000);
        let wave_id = client.open_wave(&program_id);
        
        let contributor = Address::generate(&env);
        client.record_contribution(&wave_id, &contributor, &50);
        
        let records = client.contributor_record(&contributor);
        assert_eq!(records.len(), 1);
        assert_eq!(records.get(0).unwrap().points, 50);
    }
}
