#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

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
#[should_panic(expected = "Already initialized")]
fn test_double_initialize_fails() {
    let env = Env::default();
    let (client, admin, settlement) = setup(&env);
    client.initialize(&admin, &settlement);
}

#[test]
fn test_register_and_get_program() {
    let env = Env::default();
    let (client, _admin, _settlement) = setup(&env);
    
    let program_id = 1u64;
    let name = symbol_short!("test");
    let creator = Address::generate(&env);
    
    client.register_program(&program_id, &name, &creator);
    
    let program = client.get_program(&program_id).unwrap();
    assert_eq!(program.program_id, program_id);
    assert_eq!(program.name, name);
    assert_eq!(program.admin, creator);
}

#[test]
fn test_wave_lifecycle() {
    let env = Env::default();
    let (client, _admin, _settlement) = setup(&env);
    
    let program_id = 1u64;
    client.register_program(&program_id, &symbol_short!("test"), &Address::generate(&env));
    
    // Open wave
    let wave_id = client.open_wave(&program_id);
    assert_eq!(wave_id, 1);
    
    let wave = client.get_wave(&wave_id).unwrap();
    assert_eq!(wave.status, WaveStatus::Open);
    
    // Close wave
    client.close_wave(&wave_id, &1000);
    let wave = client.get_wave(&wave_id).unwrap();
    assert_eq!(wave.status, WaveStatus::Closed);
    assert_eq!(wave.total_points, 1000);

    // Test 3: Attempt to close internal/non-existent (should fail)
    // Handled by return Result, so we check for error
    let result = client.try_close_wave(&wave_id, &1000);
    assert!(result.is_err());
}

#[test]
fn test_one_open_wave_per_program() {
    let env = Env::default();
    let (client, _admin, _settlement) = setup(&env);
    
    let program_id = 1u64;
    client.register_program(&program_id, &symbol_short!("test"), &Address::generate(&env));
    
    // Open first wave
    client.open_wave(&program_id);
    
    // Attempt to open second wave (should fail)
    let result = client.try_open_wave(&program_id);
    assert!(result.is_err());
    
    // Close first wave
    client.close_wave(&1, &1000);
    
    // Now opening a second wave should work
    let wave_id_2 = client.open_wave(&program_id);
    assert_eq!(wave_id_2, 2);
}

#[test]
fn test_contribution_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, settlement) = setup(&env);
    
    let program_id = 1u64;
    client.register_program(&program_id, &symbol_short!("test"), &Address::generate(&env));
    let wave_id = client.open_wave(&program_id);
    
    let contributor = Address::generate(&env);
    
    // Record contribution
    client.record_contribution(&wave_id, &contributor, &100);
    
    let history = client.contributor_record(&contributor);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().points, 100);
}
