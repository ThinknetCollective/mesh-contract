#![cfg(test)]

use soroban_sdk::{Address, Env, String};

#[test]
fn test_registry_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, registry::RegistryContract);
    let client = registry::RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let program_count = client.get_program_count();
    assert_eq!(program_count, 0);
}

#[test]
fn test_register_program() {
    let env = Env::default();
    let contract_id = env.register_contract(None, registry::RegistryContract);
    let client = registry::RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let program_id = String::from_str(&env, "test_program");
    let name = String::from_str(&env, "Test Program");
    let description = String::from_str(&env, "A test program");
    let creator = Address::generate(&env);
    let escrow_contract = Address::generate(&env);

    client.register_program(&program_id, &name, &description, &creator, &escrow_contract);

    let program_count = client.get_program_count();
    assert_eq!(program_count, 1);

    let program = client.get_program(&program_id);
    assert!(program.is_some());
}

#[test]
fn test_get_program() {
    let env = Env::default();
    let contract_id = env.register_contract(None, registry::RegistryContract);
    let client = registry::RegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let program_id = String::from_str(&env, "test_program");
    let name = String::from_str(&env, "Test Program");
    let description = String::from_str(&env, "A test program");
    let creator = Address::generate(&env);
    let escrow_contract = Address::generate(&env);

    client.register_program(&program_id, &name, &description, &creator, &escrow_contract);

    let program = client.get_program(&program_id);
    assert!(program.is_some());

    let non_existent = client.get_program(&String::from_str(&env, "non_existent"));
    assert!(non_existent.is_none());
}
