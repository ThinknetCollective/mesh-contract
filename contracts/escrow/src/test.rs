#![cfg(test)]

use soroban_sdk::{Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry_contract = Address::generate(&env);
    let settlement_contract = Address::generate(&env);
    
    client.initialize(&admin, &registry_contract, &settlement_contract);

    let retrieved_admin = client.get_admin();
    assert_eq!(retrieved_admin, admin);
    
    let retrieved_registry = client.get_registry_contract();
    assert_eq!(retrieved_registry, registry_contract);
    
    let retrieved_settlement = client.get_settlement_contract();
    assert_eq!(retrieved_settlement, settlement_contract);
}

#[test]
fn test_open_wave() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry_contract = Address::generate(&env);
    let settlement_contract = Address::generate(&env);
    client.initialize(&admin, &registry_contract, &settlement_contract);

    let wave_id = String::from_str(&env, "test_wave");
    let program_id = String::from_str(&env, "test_program");
    let creator = Address::generate(&env);
    let amount = 1000000i128;

    client.open_wave(&wave_id, &program_id, &creator, &amount);

    let wave_count = client.get_wave_count();
    assert_eq!(wave_count, 1);

    let wave = client.get_wave(&wave_id);
    assert!(wave.is_some());
}

#[test]
fn test_fund_wave() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry_contract = Address::generate(&env);
    let settlement_contract = Address::generate(&env);
    client.initialize(&admin, &registry_contract, &settlement_contract);

    let wave_id = String::from_str(&env, "test_wave");
    let program_id = String::from_str(&env, "test_program");
    let creator = Address::generate(&env);
    let amount = 1000000i128;

    client.open_wave(&wave_id, &program_id, &creator, &amount);

    let funder = Address::generate(&env);
    let fund_amount = 5000000i128;
    client.fund_wave(&wave_id, &funder, &fund_amount);

    let wave = client.get_wave(&wave_id);
    assert!(wave.is_some());
    let (_, _, funded_amount, _) = wave.unwrap();
    assert_eq!(funded_amount, amount + fund_amount);
}

#[test]
fn test_release_successful() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry_contract = Address::generate(&env);
    let settlement_contract = Address::generate(&env);
    client.initialize(&admin, &registry_contract, &settlement_contract);

    let wave_id = String::from_str(&env, "test_wave");
    let program_id = String::from_str(&env, "test_program");
    let creator = Address::generate(&env);
    let amount = 1000000i128;

    client.open_wave(&wave_id, &program_id, &creator, &amount);

    let funder = Address::generate(&env);
    let fund_amount = 5000000i128;
    client.fund_wave(&wave_id, &funder, &fund_amount);

    let recipient = Address::generate(&env);
    let release_amount = 2000000i128;
    
    // Mock the settlement contract authorization
    env.mock_all_auths();
    
    client.release(&wave_id, &recipient, &release_amount);

    let wave = client.get_wave(&wave_id);
    assert!(wave.is_some());
    let (_, _, remaining_amount, _) = wave.unwrap();
    assert_eq!(remaining_amount, amount + fund_amount - release_amount);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_release_unauthorized_not_initialized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let wave_id = String::from_str(&env, "test_wave");
    let recipient = Address::generate(&env);
    let release_amount = 1000i128;
    
    env.mock_all_auths();
    
    client.release(&wave_id, &recipient, &release_amount);
}

#[test]
#[should_panic(expected = "Insufficient funds in escrow")]
fn test_release_insufficient_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry_contract = Address::generate(&env);
    let settlement_contract = Address::generate(&env);
    client.initialize(&admin, &registry_contract, &settlement_contract);

    let wave_id = String::from_str(&env, "test_wave");
    let program_id = String::from_str(&env, "test_program");
    let creator = Address::generate(&env);
    let amount = 1000000i128;

    client.open_wave(&wave_id, &program_id, &creator, &amount);

    let funder = Address::generate(&env);
    let fund_amount = 5000000i128;
    client.fund_wave(&wave_id, &funder, &fund_amount);

    let recipient = Address::generate(&env);
    let release_amount = 10000000i128; // More than available
    
    env.mock_all_auths();
    
    client.release(&wave_id, &recipient, &release_amount);
}
