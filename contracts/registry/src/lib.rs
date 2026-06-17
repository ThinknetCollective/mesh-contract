#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String};

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Vec,
};

/// Represents a contributor's contribution record for a specific wave.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveContribution {
    pub wave_id: u64,
    pub points: u32,
    pub timestamp: u64,
}

/// Storage keys for the registry contract state.
#[contracttype]
pub enum DataKey {
    Contributions(Address, u64), // (contributor_address, wave_id) -> WaveContribution
    History(Address),            // contributor_address -> Vec<wave_id>
    SettlementContract,          // Address of authorized settlement contract
    Admin,                       // Address of the registry administrator
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the registry with admin and escrow contract addresses
    pub fn initialize(env: Env, admin: Address, escrow_contract: Address) {
        if env.storage().instance().has(&String::from_str(&env, "admin")) {
            panic!("already initialized");
        }
        
        env.storage().instance().set(&String::from_str(&env, "admin"), &admin);
        env.storage().instance().set(&String::from_str(&env, "escrow_contract"), &escrow_contract);
    }

    /// Register a new Wave Program
    pub fn register_program(
        env: Env,
        program_id: String,
        creator: Address,
        metadata: String,
        funding_target: u128,
    ) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized");

        admin.require_auth();

        let mut programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        if programs.contains_key(program_id.clone()) {
            panic!("program already exists");
        }

        programs.set(program_id.clone(), (creator, metadata, funding_target, false));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "programs"), &programs);
    }

    /// Get program details
    pub fn get_program(env: Env, program_id: String) -> Option<(Address, String, u128, bool)> {
        let programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        programs.get(program_id)
    }

    /// Settle a program (mark as completed)
    pub fn settle_program(env: Env, program_id: String) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized");

        admin.require_auth();

        let mut programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        let program = programs
            .get(program_id.clone())
            .expect("program not found");

        programs.set(program_id.clone(), (program.0, program.1, program.2, true));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "programs"), &programs);
    }

    /// Get escrow contract address
    pub fn get_escrow_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "escrow_contract"))
            .expect("not initialized")
    /// Initialize the contract with an admin and the authorized settlement contract address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address, settlement_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SettlementContract, &settlement_contract);
    }

    /// Update the authorized settlement contract address. Only callable by the administrator.
    pub fn set_settlement(env: Env, new_settlement: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::SettlementContract, &new_settlement);
    }

    /// Record a contribution points entry. Stores (address, wave_id) -> points.
    /// Only callable by the authorized settlement contract.
    pub fn record_contribution(
        env: Env,
        wave_id: u64,
        address: Address,
        points: u32,
    ) {
        let settlement: Address = env
            .storage()
            .instance()
            .get(&DataKey::SettlementContract)
            .expect("Settlement contract not set");
        
        // Ensure that only the authorized settlement contract has signed/authorized this call
        settlement.require_auth();

        // Create the contribution record
        let contribution = WaveContribution {
            wave_id,
            points,
            timestamp: env.ledger().timestamp(),
        };

        // Key format: CONTRIBUTIONS map with composite key (Address, u64)
        let contribution_key = DataKey::Contributions(address.clone(), wave_id);
        env.storage().persistent().set(&contribution_key, &contribution);

        // Keep track of the wave IDs for history querying
        let history_key = DataKey::History(address.clone());
        let mut history: Vec<u64> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or_else(|| Vec::new(&env));

        if !history.contains(wave_id) {
            history.push_back(wave_id);
            env.storage().persistent().set(&history_key, &history);
        }
    }

    /// Returns the full contribution history for a contributor as Vec<WaveContribution>.
    /// Handles contributors with no prior history gracefully by returning an empty Vec.
    pub fn contributor_record(env: Env, address: Address) -> Vec<WaveContribution> {
        let history_key = DataKey::History(address.clone());
        let history_res = env.storage().persistent().get::<_, Vec<u64>>(&history_key);

        match history_res {
            Some(wave_ids) => {
                let mut contributions = Vec::new(&env);
                for wave_id in wave_ids.iter() {
                    let key = DataKey::Contributions(address.clone(), wave_id);
                    if let Some(contribution) = env.storage().persistent().get::<_, WaveContribution>(&key) {
                        contributions.push_back(contribution);
                    }
                }
                contributions
            }
            None => Vec::new(&env),
        }
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized")
    }

    /// Get the current settlement contract address.
    pub fn get_settlement(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::SettlementContract)
            .expect("Contract not initialized")
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env,
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
    #[should_panic(expected = "Contract already initialized")]
    fn test_double_initialize_fails() {
        let env = Env::default();
        let (client, admin, settlement) = setup(&env);
        client.initialize(&admin, &settlement);
    }

    #[test]
    fn test_set_settlement_by_admin() {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin, _) = setup(&env);
        let new_settlement = Address::generate(&env);
        
        client.set_settlement(&new_settlement);
        assert_eq!(client.get_settlement(), new_settlement);
    }

    #[test]
    #[should_panic] // should fail require_auth validation since caller is not admin
    fn test_set_settlement_by_non_admin_fails() {
        let env = Env::default();
        // Do not mock auths, or set mock to a different caller to force failure
        let (client, _, _) = setup(&env);
        let new_settlement = Address::generate(&env);
        
        // This will panic as admin auth is missing
        client.set_settlement(&new_settlement);
    }

    #[test]
    fn test_record_single_contribution() {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, _, _) = setup(&env);
        let contributor = Address::generate(&env);
        
        env.ledger().with_mut(|li| {
            li.timestamp = 1718540000;
        });
        
        client.record_contribution(&1, &contributor, &100);
        
        let history = client.contributor_record(&contributor);
        assert_eq!(history.len(), 1);
        
        let record = history.get(0).unwrap();
        assert_eq!(record.wave_id, 1);
        assert_eq!(record.points, 100);
        assert_eq!(record.timestamp, 1718540000);
    }

    #[test]
    fn test_record_across_multiple_waves() {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, _, _) = setup(&env);
        let contributor = Address::generate(&env);
        
        // Wave 1
        env.ledger().with_mut(|li| {
            li.timestamp = 1718540000;
        });
        client.record_contribution(&1, &contributor, &150);
        
        // Wave 2
        env.ledger().with_mut(|li| {
            li.timestamp = 1718550000;
        });
        client.record_contribution(&2, &contributor, &300);
        
        let history = client.contributor_record(&contributor);
        assert_eq!(history.len(), 2);
        
        let rec1 = history.get(0).unwrap();
        assert_eq!(rec1.wave_id, 1);
        assert_eq!(rec1.points, 150);
        assert_eq!(rec1.timestamp, 1718540000);
        
        let rec2 = history.get(1).unwrap();
        assert_eq!(rec2.wave_id, 2);
        assert_eq!(rec2.points, 300);
        assert_eq!(rec2.timestamp, 1718550000);
    }

    #[test]
    fn test_contributor_no_history_returns_empty_vec() {
        let env = Env::default();
        let (client, _, _) = setup(&env);
        let contributor = Address::generate(&env);
        
        let history = client.contributor_record(&contributor);
        assert_eq!(history.len(), 0);
    }

    #[test]
    #[should_panic] // should fail require_auth of the settlement contract
    fn test_unauthorized_caller_fails() {
        let env = Env::default();
        let (client, _, _) = setup(&env);
        let contributor = Address::generate(&env);
        
        // We do not mock all auths or authorize settlement. Calling directly will panic
        client.record_contribution(&1, &contributor, &100);
    }
}
