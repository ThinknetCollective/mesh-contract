#![cfg(test)]

use soroban_sdk::{Address, Env, String};

#[test]
fn test_settlement_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, settlement::SettlementContract);
    let client = settlement::SettlementContractClient::new(&env, &contract_id);

    let escrow = Address::generate(&env);
    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&escrow, &registry, &admin);

    let settlement_count = client.get_settlement_count();
    assert_eq!(settlement_count, 0);
}

#[test]
fn test_create_settlement() {
    let env = Env::default();
    let contract_id = env.register_contract(None, settlement::SettlementContract);
    let client = settlement::SettlementContractClient::new(&env, &contract_id);

    let escrow = Address::generate(&env);
    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&escrow, &registry, &admin);

    let settlement_id = String::from_str(&env, "test_settlement");
    let wave_id = String::from_str(&env, "test_wave");
    let recipient = Address::generate(&env);
    let amount = 1000000i128;
    let proposer = Address::generate(&env);

    client.create_settlement(&settlement_id, &wave_id, &recipient, &amount, &proposer);

    let settlement_count = client.get_settlement_count();
    assert_eq!(settlement_count, 1);

    let settlement = client.get_settlement(&settlement_id);
    assert!(settlement.is_some());
}

#[test]
fn test_approve_settlement() {
    let env = Env::default();
    let contract_id = env.register_contract(None, settlement::SettlementContract);
    let client = settlement::SettlementContractClient::new(&env, &contract_id);

    let escrow = Address::generate(&env);
    let registry = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init(&escrow, &registry, &admin);

    let settlement_id = String::from_str(&env, "test_settlement");
    let wave_id = String::from_str(&env, "test_wave");
    let recipient = Address::generate(&env);
    let amount = 1000000i128;
    let proposer = Address::generate(&env);

    client.create_settlement(&settlement_id, &wave_id, &recipient, &amount, &proposer);

    client.approve_settlement(&settlement_id);

    let settlement = client.get_settlement(&settlement_id);
    assert!(settlement.is_some());
    let (_, _, _, status) = settlement.unwrap();
    assert_eq!(status, 1); // Approved status
}
