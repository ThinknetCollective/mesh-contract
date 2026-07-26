#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events, Ledger}, Address, Env, String};
use crate::interfaces::types::ProgramConfig;

fn setup(env: &Env) -> (RegistryContractClient<'static>, Address, Address) {
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let settlement = Address::generate(env);
    
    client.try_initialize(&admin, &settlement).unwrap().unwrap();
    (client, admin, settlement)
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let (client, admin, settlement) = setup(&env);
    
    assert_eq!(client.get_admin().unwrap(), admin);
    assert_eq!(client.get_settlement().unwrap(), settlement);
}

#[test]
fn test_program_registration_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    
    let config = ProgramConfig {
        name: String::from_str(&env, "Program 1"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta 1"),
        funding_target: 5000,
    };
    
    let program_id = client.try_register_program(&admin, &config).unwrap().unwrap();
    assert_eq!(program_id, 1);
    
    let stored = client.get_program(&program_id).unwrap();
    assert_eq!(stored.name, config.name);
    assert_eq!(stored.organizer, config.organizer);
}

#[test]
fn test_duplicate_program_name_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    
    let config1 = ProgramConfig {
        name: String::from_str(&env, "Same Name"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta 1"),
        funding_target: 1000,
    };
    let admin = client.get_admin();
    client.try_register_program(&admin, &config1).unwrap().unwrap();
    
    let config2 = ProgramConfig {
        name: String::from_str(&env, "Same Name"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta 2"),
        funding_target: 2000,
    };
    let result = client.try_register_program(&admin, &config2).unwrap();
    assert_eq!(result, Err(Ok(ContractError::ProgramNameExists)));
}

#[test]
fn test_registration_by_onboarder() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    
    let onboarder = Address::generate(&env);
    client.try_set_onboarder(&onboarder).unwrap().unwrap();
    
    let config = ProgramConfig {
        name: String::from_str(&env, "Onboarded"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta"),
        funding_target: 1000,
    };
    
    let id = client.try_register_program(&onboarder, &config).unwrap().unwrap();
    assert_eq!(id, 1);
}

#[test]
fn test_unauthorized_registration_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    
    let config = ProgramConfig {
        name: String::from_str(&env, "Unauthorized"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta"),
        funding_target: 1000,
    };
    
    let someone_else = Address::generate(&env);
    let result = client.try_register_program(&someone_else, &config).unwrap();
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_full_wave_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    
    // 1. Register Program
    let config = ProgramConfig {
        name: String::from_str(&env, "Wave Program"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta"),
        funding_target: 1000,
    };
    let admin = client.get_admin();
    let program_id = client.register_program(&admin, &config);
    
    // 2. Open Wave
    let open_ts = 200000;
    env.ledger().with_mut(|li| li.timestamp = open_ts);
    let wave_id = client.open_wave(&program_id);
    assert_eq!(wave_id, 1);
    
    let wave = client.get_wave(&wave_id).expect("Wave should exist");
    assert_eq!(wave.status, WaveStatus::Open);
    assert_eq!(wave.program_id, program_id);
    assert_eq!(wave.difficulty_level, 1); // Default initial difficulty
    
    // 3. Close Wave
    let close_ts = 300000;
    env.ledger().with_mut(|li| li.timestamp = close_ts);
    client.try_close_wave(&wave_id, &1500).unwrap().unwrap();
    
    let wave_after = client.get_wave(&wave_id).expect("Wave should exist");
    assert_eq!(wave_after.status, WaveStatus::Closed);
}
