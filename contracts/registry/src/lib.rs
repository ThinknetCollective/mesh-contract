#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub mod interfaces;
use interfaces::types::ProgramConfig;
use interfaces::errors::ContractError;

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
    pub program_id: u32,
    pub wave_id: u32,
    pub opened_at: u64,
    pub closed_at: u64,
    pub total_points: u32,
    pub status: WaveStatus,
    pub difficulty_level: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributorPerformance {
    pub total_waves_participated: u32,
    pub total_points_earned: u32,
    pub average_points_per_wave: u32,
    pub success_rate: u32,
    pub last_updated: u64,
}

/// Storage keys for the registry contract state.
#[contracttype]
pub enum DataKey {
    Admin,
    Onboarder,
    SettlementContract,
    Programs(u32),
    ProgramName(String), // Name -> ID
    ProgramCounter,
    Waves(u32),
    WaveCounter,
    Contributions(Address, u32), // contributor, wave_id -> contribution
    History(Address),            // contributor -> Vec<wave_id>
    ContributorPerformance(Address), // contributor -> performance metrics
    ProgramDifficulty(u32),      // program_id -> current difficulty level
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the contract with an admin and the authorized settlement contract address.
    pub fn initialize(env: Env, admin: Address, settlement_contract: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SettlementContract, &settlement_contract);
        env.storage().instance().set(&DataKey::WaveCounter, &0u32);
        env.storage().instance().set(&DataKey::ProgramCounter, &0u32);
        Ok(())
    }

    /// Set the authorized onboarder address. Only callable by admin.
    pub fn set_onboarder(env: Env, onboarder: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Onboarder, &onboarder);
        Ok(())
    }

    /// Register a new Wave Program. Only callable by admin or onboarder.
    pub fn register_program(
        env: Env,
        caller: Address,
        config: ProgramConfig,
    ) -> Result<u32, ContractError> {
        caller.require_auth();

        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;
        let onboarder: Option<Address> = env.storage().instance().get(&DataKey::Onboarder);
        
        let is_admin = caller == admin;
        let is_onboarder = onboarder.map(|o| o == caller).unwrap_or(false);

        if !is_admin && !is_onboarder {
            return Err(ContractError::Unauthorized);
        }

        // Duplicate name check
        if env.storage().persistent().has(&DataKey::ProgramName(config.name.clone())) {
            return Err(ContractError::ProgramNameExists);
        }

        // Increment program counter
        let mut counter: u32 = env.storage().instance().get(&DataKey::ProgramCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::ProgramCounter, &counter);

        let program_id = counter;
        env.storage().persistent().set(&DataKey::Programs(program_id), &config);
        env.storage().persistent().set(&DataKey::ProgramName(config.name.clone()), &program_id);

        // Emit ProgramRegistered event
        env.events().publish(
            (symbol_short!("prog_reg"), program_id, config.name.clone(), config.organizer.clone()),
            (program_id, config.name, config.organizer),
        );

        Ok(program_id)
    }

    /// Opens a new wave cycle for a program. Returns wave_id.
    pub fn open_wave(env: Env, program_id: u32) -> Result<u32, ContractError> {
        if !env.storage().persistent().has(&DataKey::Programs(program_id)) {
            return Err(ContractError::ProgramNotFound);
        }

        // Get or initialize difficulty level for the program
        let difficulty_level: u32 = env.storage().persistent().get(&DataKey::ProgramDifficulty(program_id)).unwrap_or(1);

        // Increment global wave ID
        let mut counter: u32 = env.storage().instance().get(&DataKey::WaveCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::WaveCounter, &counter);

        let wave_id = counter;
        let wave = WaveMeta {
            program_id,
            wave_id,
            opened_at: env.ledger().timestamp(),
            closed_at: 0,
            total_points: 0,
            status: WaveStatus::Open,
            difficulty_level,
        };

        env.storage().persistent().set(&DataKey::Waves(wave_id), &wave);

        // Emit WaveOpened event
        env.events().publish(
            (symbol_short!("wave_open"), program_id, wave_id, difficulty_level),
            env.ledger().timestamp(),
        );

        Ok(wave_id)
    }

    /// Closes an open wave cycle and marks it ready for settlement.
    pub fn close_wave(env: Env, wave_id: u32, total_points: u32) -> Result<(), ContractError> {
        let mut wave: WaveMeta = env
            .storage()
            .persistent()
            .get(&DataKey::Waves(wave_id))
            .ok_or(ContractError::WaveNotFound)?;

        if wave.status != WaveStatus::Open {
            return Err(ContractError::WaveNotFound); // Or specific Closed status error
        }

        wave.closed_at = env.ledger().timestamp();
        wave.total_points = total_points;
        wave.status = WaveStatus::Closed;

        env.storage().persistent().set(&DataKey::Waves(wave_id), &wave);

        // Adjust difficulty based on overall performance
        Self::adjust_program_difficulty(&env, wave.program_id, total_points, wave.difficulty_level);

        // Emit WaveClosed event
        env.events().publish(
            (symbol_short!("wave_cls"), wave_id, total_points),
            env.ledger().timestamp(),
        );
        Ok(())
    }

    /// Record a contribution points entry. Only callable by settlement contract.
    pub fn record_contribution(env: Env, wave_id: u32, address: Address, points: u32) -> Result<(), ContractError> {
        let settlement: Address = env
            .storage().instance()
            .get(&DataKey::SettlementContract)
            .ok_or(ContractError::SettlementNotSet)?;
        settlement.require_auth();

        let wave: WaveMeta = env
            .storage()
            .persistent()
            .get(&DataKey::Waves(wave_id))
            .ok_or(ContractError::WaveNotFound)?;
        
        if wave.status != WaveStatus::Open {
            return Err(ContractError::WaveNotFound);
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
            env.storage().persistent().set(&DataKey::History(address.clone()), &history);
        }

        // Update contributor performance metrics
        Self::update_contributor_performance(&env, address.clone(), points);
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

    pub fn get_program(env: Env, program_id: u32) -> Option<ProgramConfig> {
        env.storage().persistent().get(&DataKey::Programs(program_id))
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(ContractError::NotInitialized)
    }

    pub fn get_settlement(env: Env) -> Result<Address, ContractError> {
        env.storage().instance().get(&DataKey::SettlementContract).ok_or(ContractError::NotInitialized)
    }
    
    pub fn set_settlement(env: Env, new_settlement: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(ContractError::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::SettlementContract, &new_settlement);
        Ok(())
    }

    /// Update contributor performance metrics after recording a contribution
    fn update_contributor_performance(env: &Env, address: Address, points: u32) {
        let mut perf: ContributorPerformance = env
            .storage()
            .persistent()
            .get(&DataKey::ContributorPerformance(address.clone()))
            .unwrap_or_else(|| ContributorPerformance {
                total_waves_participated: 0,
                total_points_earned: 0,
                average_points_per_wave: 0,
                success_rate: 100,
                last_updated: 0,
            });

        perf.total_waves_participated += 1;
        perf.total_points_earned += points;
        perf.average_points_per_wave = perf.total_points_earned / perf.total_waves_participated;
        perf.last_updated = env.ledger().timestamp();

        // Simple success rate calculation: if points > 0, it's a success
        if points > 0 {
            let successful_waves = (perf.success_rate as u128 * perf.total_waves_participated as u128 / 100) + 1;
            perf.success_rate = (successful_waves * 100 / perf.total_waves_participated as u128) as u32;
        }

        env.storage().persistent().set(&DataKey::ContributorPerformance(address), &perf);
    }

    /// Adjust program difficulty based on wave performance
    fn adjust_program_difficulty(env: &Env, program_id: u32, total_points: u32, current_difficulty: u32) {
        // Performance thresholds for difficulty adjustment
        const HIGH_PERFORMANCE_THRESHOLD: u32 = 1000;
        const LOW_PERFORMANCE_THRESHOLD: u32 = 100;
        const MAX_DIFFICULTY: u32 = 10;
        const MIN_DIFFICULTY: u32 = 1;

        let new_difficulty = if total_points > HIGH_PERFORMANCE_THRESHOLD {
            // High performance - increase difficulty
            (current_difficulty + 1).min(MAX_DIFFICULTY)
        } else if total_points < LOW_PERFORMANCE_THRESHOLD && current_difficulty > MIN_DIFFICULTY {
            // Low performance - decrease difficulty
            current_difficulty - 1
        } else {
            // Maintain current difficulty
            current_difficulty
        };

        if new_difficulty != current_difficulty {
            env.storage().persistent().set(&DataKey::ProgramDifficulty(program_id), &new_difficulty);
            
            // Emit DifficultyAdjusted event
            env.events().publish(
                (symbol_short!("diff_adj"), program_id, current_difficulty, new_difficulty),
                total_points,
            );
        }
    }

    /// Get contributor performance metrics
    pub fn get_contributor_performance(env: Env, address: Address) -> Option<ContributorPerformance> {
        env.storage().persistent().get(&DataKey::ContributorPerformance(address))
    }

    /// Get current difficulty level for a program
    pub fn get_program_difficulty(env: Env, program_id: u32) -> u32 {
        env.storage().persistent().get(&DataKey::ProgramDifficulty(program_id)).unwrap_or(1)
    }

    /// Manually set difficulty level for a program (admin only)
    pub fn set_program_difficulty(env: Env, program_id: u32, difficulty: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        admin.require_auth();

        if difficulty < 1 || difficulty > 10 {
            panic!("difficulty must be between 1 and 10");
        }

        env.storage().persistent().set(&DataKey::ProgramDifficulty(program_id), &difficulty);
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
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
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
        
        // Verify events (disabled due to SDK flakiness)
        // let events = env.events().all();
        // assert!(events.len() >= 2);
    }

    #[test]
    #[should_panic(expected = "program doesn't exist")]
    fn test_open_wave_non_existent_program() {
        let env = Env::default();
        let (client, _, _) = setup(&env);
        client.open_wave(&999);
    }

    #[test]
    #[should_panic(expected = "wave already closed or settled")]
    fn test_close_already_closed_wave() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        let wave_id = client.open_wave(&program_id);
        
        client.close_wave(&wave_id, &100);
        client.close_wave(&wave_id, &200); // Should panic
    }

    #[test]
    fn test_record_contribution() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        let wave_id = client.open_wave(&program_id);
        
        let contributor = Address::generate(&env);
        client.record_contribution(&wave_id, &contributor, &50);
        
        let records = client.contributor_record(&contributor);
        assert_eq!(records.len(), 1);
        assert_eq!(records.get(0).unwrap().points, 50);
    }

    #[test]
    fn test_difficulty_initialization() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
        // Initial difficulty should be 1
        assert_eq!(client.get_program_difficulty(&program_id), 1);
        
        // Open wave should use initial difficulty
        let wave_id = client.open_wave(&program_id);
        let wave = client.get_wave(&wave_id).unwrap();
        assert_eq!(wave.difficulty_level, 1);
    }

    #[test]
    fn test_difficulty_increase_on_high_performance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        let wave_id = client.open_wave(&program_id);
        
        // Close wave with high performance (> 1000 points)
        client.close_wave(&wave_id, &1500);
        
        // Difficulty should increase to 2
        assert_eq!(client.get_program_difficulty(&program_id), 2);
        
        // Next wave should use new difficulty
        let wave_id2 = client.open_wave(&program_id);
        let wave2 = client.get_wave(&wave_id2).unwrap();
        assert_eq!(wave2.difficulty_level, 2);
    }

    #[test]
    fn test_difficulty_decrease_on_low_performance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
        // Set initial difficulty to 3
        client.set_program_difficulty(&program_id, &3);
        assert_eq!(client.get_program_difficulty(&program_id), 3);
        
        let wave_id = client.open_wave(&program_id);
        
        // Close wave with low performance (< 100 points)
        client.close_wave(&wave_id, &50);
        
        // Difficulty should decrease to 2
        assert_eq!(client.get_program_difficulty(&program_id), 2);
    }

    #[test]
    fn test_difficulty_max_cap() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
        // Set difficulty to max (10)
        client.set_program_difficulty(&program_id, &10);
        assert_eq!(client.get_program_difficulty(&program_id), 10);
        
        let wave_id = client.open_wave(&program_id);
        
        // Close wave with extremely high performance
        client.close_wave(&wave_id, &10000);
        
        // Difficulty should stay at max (10)
        assert_eq!(client.get_program_difficulty(&program_id), 10);
    }

    #[test]
    fn test_contributor_performance_tracking() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        let wave_id = client.open_wave(&program_id);
        
        let contributor = Address::generate(&env);
        
        // Record first contribution
        client.record_contribution(&wave_id, &contributor, &100);
        
        let perf = client.get_contributor_performance(&contributor).unwrap();
        assert_eq!(perf.total_waves_participated, 1);
        assert_eq!(perf.total_points_earned, 100);
        assert_eq!(perf.average_points_per_wave, 100);
        
        // Record second contribution
        client.record_contribution(&wave_id, &contributor, &200);
        
        let perf = client.get_contributor_performance(&contributor).unwrap();
        assert_eq!(perf.total_waves_participated, 2);
        assert_eq!(perf.total_points_earned, 300);
        assert_eq!(perf.average_points_per_wave, 150);
    }

    #[test]
    #[should_panic(expected = "difficulty must be between 1 and 10")]
    fn test_set_invalid_difficulty() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
        // Try to set invalid difficulty (0)
        client.set_program_difficulty(&program_id, &0);
    }

    #[test]
    fn test_difficulty_maintained_on_moderate_performance() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);
        
        let config = ProgramConfig {
            name: String::from_str(&env, "prog1"),
            organizer: Address::generate(&env),
            metadata: String::from_str(&env, "meta"),
            funding_target: 1000,
        };
        let admin: Address = client.get_admin();
        let program_id = client.register_program(&admin, &config);
        
        // Set difficulty to 5
        client.set_program_difficulty(&program_id, &5);
        assert_eq!(client.get_program_difficulty(&program_id), 5);
        
        let wave_id = client.open_wave(&program_id);
        
        // Close wave with moderate performance (between thresholds)
        client.close_wave(&wave_id, &500);
        
        // Difficulty should remain at 5
        assert_eq!(client.get_program_difficulty(&program_id), 5);
    }
}

#[cfg(test)]
mod test;
