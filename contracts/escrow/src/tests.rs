#![cfg(test)]

use soroban_sdk::{Symbol, Address, Env};

#[test]
fn test_escrow_fund_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "test_program");
    let amount: i128 = 1000;

    // Call the fund function
    client.fund(&program_id, &amount);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;
    
    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'escrow_funded'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "escrow_funded", "Event topic should be 'escrow_funded'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_escrow_release_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let recipient = Address::generate(&env);
    let amount: i128 = 500;

    // Call the release function
    client.release(&wave_id, &recipient, &amount);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'escrow_released'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "escrow_released", "Event topic should be 'escrow_released'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_escrow_fund_with_zero_amount_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "test_program");
    let amount: i128 = 0;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.fund(&program_id, &amount);
    }));

    assert!(result.is_err(), "Expected panic for zero amount");
}

#[test]
fn test_escrow_release_with_negative_amount_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let recipient = Address::generate(&env);
    let amount: i128 = -100;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.release(&wave_id, &recipient, &amount);
    }));

    assert!(result.is_err(), "Expected panic for negative amount");
}

#[test]
fn test_escrow_fund_emits_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let program_id = Symbol::new(&env, "test_program");
    let amount: i128 = 2000;

    client.fund(&program_id, &amount);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    // Verify event data contains program_id and amount
    assert!(event.data.to_string().contains("test_program"), 
            "Event data should contain program_id");
}

#[test]
fn test_multiple_escrow_operations_emit_multiple_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::EscrowContract);

    let client = super::EscrowContractClient::new(&env, &contract_id);

    let program_id1 = Symbol::new(&env, "program_1");
    let program_id2 = Symbol::new(&env, "program_2");

    client.fund(&program_id1, &1000);
    client.fund(&program_id2, &2000);

    let events = env.events().all();
    assert_eq!(events.len(), 2, "Expected two events from two fund calls");
}
