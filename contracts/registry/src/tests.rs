#![cfg(test)]

use soroban_sdk::{Symbol, Address, Env};

#[test]
fn test_registry_register_program_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "test_program");
    let name = Symbol::new(&env, "Test Program");
    let organizer = Address::generate(&env);

    // Call the register_program function
    client.register_program(&program_id, &name, &organizer);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'registry_program_registered'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "registry_program_registered", 
                   "Event topic should be 'registry_program_registered'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_registry_open_wave_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let program_id = Symbol::new(&env, "test_program");

    // Call the open_wave function
    client.open_wave(&wave_id, &program_id);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'registry_wave_opened'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "registry_wave_opened", "Event topic should be 'registry_wave_opened'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_registry_close_wave_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = 1000;

    // Call the close_wave function
    client.close_wave(&wave_id, &total_points);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'registry_wave_closed'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "registry_wave_closed", "Event topic should be 'registry_wave_closed'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_registry_open_wave_with_empty_wave_id_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "");
    let program_id = Symbol::new(&env, "test_program");

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.open_wave(&wave_id, &program_id);
    }));

    assert!(result.is_err(), "Expected panic for empty wave_id");
}

#[test]
fn test_registry_close_wave_with_negative_points_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = -100;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.close_wave(&wave_id, &total_points);
    }));

    assert!(result.is_err(), "Expected panic for negative points");
}

#[test]
fn test_registry_program_registered_emits_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "program_001");
    let name = Symbol::new(&env, "Education Initiative");
    let organizer = Address::generate(&env);

    client.register_program(&program_id, &name, &organizer);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    // Verify event data contains program_id and name
    let event_str = event.data.to_string();
    assert!(event_str.contains("program_001"), "Event data should contain program_id");
}

#[test]
fn test_registry_wave_opened_emits_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "wave_001");
    let program_id = Symbol::new(&env, "program_001");

    client.open_wave(&wave_id, &program_id);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    // Verify event data contains wave_id and program_id
    let event_str = event.data.to_string();
    assert!(event_str.contains("wave_001"), "Event data should contain wave_id");
}

#[test]
fn test_registry_wave_closed_emits_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "wave_001");
    let total_points: i128 = 5000;

    client.close_wave(&wave_id, &total_points);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    // Verify event contains correct data
    let event_str = event.data.to_string();
    assert!(event_str.contains("wave_001"), "Event data should contain wave_id");
}

#[test]
fn test_registry_update_program_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "test_program");
    let name = Symbol::new(&env, "Updated Program");
    let organizer = Address::generate(&env);

    // Call the update_program function
    client.update_program(&program_id, &name, &organizer);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Verify the event topic is 'registry_program_registered'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "registry_program_registered", 
                   "Update should emit registry_program_registered event");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_multiple_registry_operations_emit_multiple_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::RegistryContract);

    let client = super::RegistryContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "program_1");
    let name = Symbol::new(&env, "Program 1");
    let organizer = Address::generate(&env);
    let wave_id = Symbol::new(&env, "wave_1");

    client.register_program(&program_id, &name, &organizer);
    client.open_wave(&wave_id, &program_id);
    client.close_wave(&wave_id, &1000);

    let events = env.events().all();
    assert_eq!(events.len(), 3, "Expected three events from three operations");
}
