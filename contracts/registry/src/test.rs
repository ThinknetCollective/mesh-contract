#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events, Ledger}, Address, Env, String};
use crate::interfaces::types::ProgramConfig;

fn setup(env: &Env) -> (RegistryContractClient<'static>, Address, Address) {
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let settlement = Address::generate(env);
    
    client.initialize(&admin, &settlement);
    (client, admin, settlement)
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let (client, admin, settlement) = setup(&env);
    
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_settlement(), settlement);
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
    
    let program_id = client.register_program(&admin, &config);
    assert_eq!(program_id, 1);
    
    let stored = client.get_program(&program_id).unwrap();
    assert_eq!(stored.name, config.name);
    assert_eq!(stored.organizer, config.organizer);
}

#[test]
#[should_panic(expected = "program name already exists")]
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
    client.register_program(&admin, &config1);
    
    let config2 = ProgramConfig {
        name: String::from_str(&env, "Same Name"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta 2"),
        funding_target: 2000,
    };
    client.register_program(&admin, &config2);
}

#[test]
fn test_registration_by_onboarder() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _) = setup(&env);
    
    let onboarder = Address::generate(&env);
    client.set_onboarder(&onboarder);
    
    let config = ProgramConfig {
        name: String::from_str(&env, "Onboarded"),
        organizer: Address::generate(&env),
        metadata: String::from_str(&env, "Meta"),
        funding_target: 1000,
    };
    
    let id = client.register_program(&onboarder, &config);
    assert_eq!(id, 1);
}

#[test]
#[should_panic(expected = "unauthorized")]
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
    client.register_program(&someone_else, &config);
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
    
    // 3. Close Wave
    let close_ts = 300000;
    env.ledger().with_mut(|li| li.timestamp = close_ts);
    client.close_wave(&wave_id, &1500);
    
    let wave_after = client.get_wave(&wave_id).expect("Wave should exist");
    assert_eq!(wave_after.status, WaveStatus::Closed);
}
