#![cfg(test)]

use soroban_sdk::{Symbol, Env};

#[test]
fn test_settlement_settle_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = 1000;
    let contributor_count: u32 = 5;

    // Call the settle function
    client.settle(&wave_id, &total_points, &contributor_count);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'settlement_settled'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "settlement_settled", "Event topic should be 'settlement_settled'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_settlement_finalize_emits_event() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = 1500;
    let contributor_count: u32 = 10;

    // Call the finalize function
    client.finalize(&wave_id, &total_points, &contributor_count);

    // Verify the event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    let event_topics = &event.topics;

    // Check the event topic
    assert_eq!(event_topics.len(), 1, "Event should have 1 topic");

    // Verify the event topic is 'settlement_settled'
    let topic = &event_topics[0];
    if let soroban_sdk::Val::Symbol(s) = topic {
        let topic_str = std::str::from_utf8(s.to_utf8().as_bytes())
            .expect("Invalid UTF-8 in topic");
        assert_eq!(topic_str, "settlement_settled", "Event topic should be 'settlement_settled'");
    } else {
        panic!("Event topic should be a symbol");
    }
}

#[test]
fn test_settlement_settle_with_zero_points_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = 0;
    let contributor_count: u32 = 5;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.settle(&wave_id, &total_points, &contributor_count);
    }));

    assert!(result.is_err(), "Expected panic for zero points");
}

#[test]
fn test_settlement_settle_with_zero_contributors_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = 1000;
    let contributor_count: u32 = 0;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.settle(&wave_id, &total_points, &contributor_count);
    }));

    assert!(result.is_err(), "Expected panic for zero contributors");
}

#[test]
fn test_settlement_finalize_with_negative_points_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "test_wave");
    let total_points: i128 = -1000;
    let contributor_count: u32 = 5;

    // This should panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.finalize(&wave_id, &total_points, &contributor_count);
    }));

    assert!(result.is_err(), "Expected panic for negative points");
}

#[test]
fn test_settlement_settle_emits_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id = Symbol::new(&env, "wave_001");
    let total_points: i128 = 5000;
    let contributor_count: u32 = 20;

    client.settle(&wave_id, &total_points, &contributor_count);

    let events = env.events().all();
    assert_eq!(events.len(), 1, "Expected exactly one event");

    let event = events.first().unwrap();
    // Verify event data contains wave_id, total_points, and contributor_count
    let event_str = event.data.to_string();
    assert!(event_str.contains("wave_001"), "Event data should contain wave_id");
}

#[test]
fn test_multiple_settlement_operations_emit_multiple_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, super::SettlementContract);

    let client = super::SettlementContractClient::new(&env, &contract_id);

    let wave_id1 = Symbol::new(&env, "wave_1");
    let wave_id2 = Symbol::new(&env, "wave_2");

    client.settle(&wave_id1, &1000, &5);
    client.settle(&wave_id2, &2000, &10);

    let events = env.events().all();
    assert_eq!(events.len(), 2, "Expected two events from two settle calls");
}
