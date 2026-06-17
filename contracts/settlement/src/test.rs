#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
use crate::{ContributorPoints, SettlementContract, SettlementContractClient};

#[soroban_sdk::contract]
pub struct MockEscrow;

#[soroban_sdk::contractimpl]
impl MockEscrow {
    pub fn init(_env: Env, _registry: Address, _admin: Address) {}
    pub fn release(env: Env, _wave_id: String, _contributor: Address, amount: i128) {
        // Track released amount in temporary storage for verification
        let key = soroban_sdk::symbol_short!("rel_amt");
        let current: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current + amount));
    }
    pub fn get_released_amount(env: Env) -> i128 {
        env.storage().instance().get(&soroban_sdk::symbol_short!("rel_amt")).unwrap_or(0)
    }
}

fn setup_env() -> (Env, SettlementContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    
    let settlement_id = env.register_contract(None, SettlementContract);
    let settlement_client = SettlementContractClient::new(&env, &settlement_id);
    
    let escrow_id = env.register_contract(None, MockEscrow);
    
    let admin = Address::generate(&env);
    let registry = Address::generate(&env);
    
    settlement_client.init(&escrow_id, &registry, &admin);
    
    (env, settlement_client, escrow_id)
}

#[test]
fn test_settle_wave_even_split() {
    let (env, client, escrow_id) = setup_env();
    let wave_id = String::from_str(&env, "wave_1");
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorPoints { address: user1.clone(), points: 50 });
    contributors.push_back(ContributorPoints { address: user2.clone(), points: 50 });
    
    let reward_pool = 1000i128;
    client.settle_wave(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 1000);
}

#[test]
fn test_settle_wave_uneven_split() {
    let (env, client, escrow_id) = setup_env();
    let wave_id = String::from_str(&env, "wave_2");
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorPoints { address: user1, points: 10 });
    contributors.push_back(ContributorPoints { address: user2, points: 20 });
    contributors.push_back(ContributorPoints { address: user3, points: 30 });
    
    // Total points = 60
    // Rewards: 10/60 * 600 = 100, 20/60 * 600 = 200, 30/60 * 600 = 300
    let reward_pool = 600i128;
    client.settle_wave(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 600);
}

#[test]
fn test_settle_wave_single_contributor() {
    let (env, client, escrow_id) = setup_env();
    let wave_id = String::from_str(&env, "wave_3");
    
    let user1 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorPoints { address: user1, points: 100 });
    
    let reward_pool = 1000i128;
    client.settle_wave(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 1000);
}

#[test]
fn test_settle_wave_dust_handling() {
    let (env, client, escrow_id) = setup_env();
    let wave_id = String::from_str(&env, "wave_4");
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorPoints { address: user1, points: 1 });
    contributors.push_back(ContributorPoints { address: user2, points: 1 });
    contributors.push_back(ContributorPoints { address: user3, points: 1 });
    
    // Total points = 3
    // Reward pool = 100
    // Each gets 1/3 * 100 = 33. Total released = 99. Dust = 1.
    let reward_pool = 100i128;
    client.settle_wave(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 99);
}

#[test]
fn test_settle_wave_arithmetic_safety() {
    let (env, client, escrow_id) = setup_env();
    let wave_id = String::from_str(&env, "wave_5");
    
    let user1 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    // Use very large points and large reward pool to check i128
    let large_points = 1_000_000_000_000_000i128;
    contributors.push_back(ContributorPoints { address: user1, points: large_points });
    
    let large_reward = 1_000_000_000_000_000_000i128;
    client.settle_wave(&wave_id, &contributors, &large_reward);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), large_reward);
}
