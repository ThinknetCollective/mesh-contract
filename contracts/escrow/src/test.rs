#![cfg(test)]

use soroban_sdk::{Address, Env, String};

#[test]
fn test_escrow_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&registry, &admin);

    let wave_count = client.get_wave_count();
    assert_eq!(wave_count, 0);
}

#[test]
fn test_open_wave() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&registry, &admin);

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

    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&registry, &admin);

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
