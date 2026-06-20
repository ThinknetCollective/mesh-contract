#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
use crate::{ContributorResult, SettlementContract, SettlementContractClient};
use crate::interfaces::errors::ContractError;

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

fn setup_env() -> (Env, SettlementContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    
    let settlement_id = env.register_contract(None, SettlementContract);
    let settlement_client = SettlementContractClient::new(&env, &settlement_id);
    
    let escrow_id = env.register_contract(None, MockEscrow);
    
    let admin = Address::generate(&env);
    let registry = Address::generate(&env);
    
    settlement_client.init(&escrow_id, &registry, &admin);
    
    (env, settlement_client, escrow_id, admin)
}

#[test]
fn test_settle_even_split() {
    let (env, client, escrow_id, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_1");
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorResult { address: user1.clone(), points: 50 });
    contributors.push_back(ContributorResult { address: user2.clone(), points: 50 });
    
    let reward_pool = 1000i128;
    client.settle(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 1000);
}

#[test]
fn test_settle_empty_results() {
    let (env, client, _, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_empty");
    let contributors = Vec::new(&env);
    let reward_pool = 1000i128;
    
    let result = client.try_settle(&wave_id, &contributors, &reward_pool);
    assert_eq!(result, Err(Ok(ContractError::EmptyResults)));
}

#[test]
fn test_settle_zero_points() {
    let (env, client, _, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_zero");
    
    let user1 = Address::generate(&env);
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorResult { address: user1, points: 0 });
    
    let reward_pool = 1000i128;
    let result = client.try_settle(&wave_id, &contributors, &reward_pool);
    assert_eq!(result, Err(Ok(ContractError::ZeroTotalPoints)));
}

#[test]
fn test_settle_duplicate_contributor() {
    let (env, client, _, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_dup");
    
    let user1 = Address::generate(&env);
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorResult { address: user1.clone(), points: 50 });
    contributors.push_back(ContributorResult { address: user1, points: 50 });
    
    let reward_pool = 1000i128;
    let result = client.try_settle(&wave_id, &contributors, &reward_pool);
    assert_eq!(result, Err(Ok(ContractError::DuplicateContributor)));
}

#[test]
fn test_settle_already_settled() {
    let (env, client, _, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_settled");
    
    let user1 = Address::generate(&env);
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorResult { address: user1, points: 100 });
    
    let reward_pool = 1000i128;
    client.settle(&wave_id, &contributors, &reward_pool);
    
    // Try to settle again
    let result = client.try_settle(&wave_id, &contributors, &reward_pool);
    assert_eq!(result, Err(Ok(ContractError::AlreadySettled)));
}

#[test]
fn test_settle_uneven_split() {
    let (env, client, escrow_id, _) = setup_env();
    let wave_id = String::from_str(&env, "wave_2");
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    
    let mut contributors = Vec::new(&env);
    contributors.push_back(ContributorResult { address: user1, points: 10 });
    contributors.push_back(ContributorResult { address: user2, points: 20 });
    contributors.push_back(ContributorResult { address: user3, points: 30 });
    
    let reward_pool = 600i128;
    client.settle(&wave_id, &contributors, &reward_pool);
    
    let mock_escrow = MockEscrowClient::new(&env, &escrow_id);
    assert_eq!(mock_escrow.get_released_amount(), 600);
}
