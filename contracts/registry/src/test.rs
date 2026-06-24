#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Events, Ledger}, Address, Env, String};

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
fn test_program_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    
    let program_id = String::from_str(&env, "program_1");
    let creator = Address::generate(&env);
    let metadata = String::from_str(&env, "Some metadata");
    let target: u128 = 5000;
    
    client.register_program(&program_id, &creator, &metadata, &target);
    
    let program = client.get_program(&program_id).unwrap();
    assert_eq!(program.creator, creator);
    assert_eq!(program.metadata, metadata);
    assert_eq!(program.funding_target, target);
    assert!(program.is_active);
}

#[test]
fn test_full_wave_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    
    // 1. Register Program
    let program_id = String::from_str(&env, "prog_lifecycle");
    client.register_program(&program_id, &Address::generate(&env), &String::from_str(&env, "meta"), &1000);
    
    // 2. Open Wave
    let open_ts = 200000;
    env.ledger().with_mut(|li| li.timestamp = open_ts);
    let wave_id = client.open_wave(&program_id);
    assert_eq!(wave_id, 1);
    
    let wave = client.get_wave(&wave_id).expect("Wave should exist");
    assert_eq!(wave.status, WaveStatus::Open);
    assert_eq!(wave.opened_at, open_ts);
    
    // 3. Close Wave
    let close_ts = 300000;
    env.ledger().with_mut(|li| li.timestamp = close_ts);
    let total_points = 1500;
    client.close_wave(&wave_id, &total_points);
    
    let wave_after = client.get_wave(&wave_id).expect("Wave should exist");
    assert_eq!(wave_after.status, WaveStatus::Closed);
    assert_eq!(wave_after.closed_at, close_ts);
    assert_eq!(wave_after.total_points, total_points);
}

#[test]
#[should_panic(expected = "wave already closed or settled")]
fn test_fail_reopen_or_reclose_wave() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _) = setup(&env);
    
    let program_id = String::from_str(&env, "prog_fail");
    client.register_program(&program_id, &Address::generate(&env), &String::from_str(&env, "meta"), &1000);
    let wave_id = client.open_wave(&program_id);
    
    client.close_wave(&wave_id, &100);
    // Attempting to close again should fail (satisfies "attempt reopen (should fail)" in spirit, 
    // as there is no specific reopen function, but closing a non-open wave is the equivalent lifecycle violation)
    client.close_wave(&wave_id, &200);
}
