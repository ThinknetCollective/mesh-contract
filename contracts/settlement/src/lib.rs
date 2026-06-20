#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, String, Vec, Map
};

mod interfaces;
use interfaces::errors::ContractError;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaveStatus {
    Pending = 0,
    Settled = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributorResult {
    pub address: Address,
    pub points: i128,
}

#[contract]
pub struct SettlementContract;

#[contractimpl]
impl SettlementContract {
    /// Initialize the settlement contract with escrow and registry addresses
    pub fn init(env: Env, escrow: Address, registry: Address, admin: Address) {
        let escrow_key = symbol_short!("escrow");
        let registry_key = symbol_short!("registry");
        let admin_key = symbol_short!("admin");
        let count_key = symbol_short!("stl_cnt");
        env.storage().instance().set(&escrow_key, &escrow);
        env.storage().instance().set(&registry_key, &registry);
        env.storage().instance().set(&admin_key, &admin);
        env.storage().instance().set(&count_key, &0u32);
    }

    /// Settle a Wave with proportional rewards and input validation
    pub fn settle(
        env: Env,
        wave_id: String,
        results: Vec<ContributorResult>,
        wave_reward_pool: i128,
    ) -> Result<(), ContractError> {
        // 1. Requirement: Reject if results is empty
        if results.is_empty() {
            return Err(ContractError::EmptyResults);
        }

        // 2. Requirement: Reject if wave is already settled
        let wave_status_key = (symbol_short!("status"), wave_id.clone());
        let wave_status: WaveStatus = env
            .storage()
            .instance()
            .get(&wave_status_key)
            .unwrap_or(WaveStatus::Pending);

        if wave_status == WaveStatus::Settled {
            return Err(ContractError::AlreadySettled);
        }

        let mut total_points: i128 = 0;
        let mut seen_contributors: Vec<Address> = Vec::new(&env);

        for item in results.iter() {
            // 3. Requirement: Reject duplicate contributor addresses
            if seen_contributors.contains(item.address.clone()) {
                return Err(ContractError::DuplicateContributor);
            }
            seen_contributors.push_back(item.address.clone());
            total_points += item.points;
        }

        // 4. Requirement: Reject if total points across all contributors equals zero
        if total_points == 0 {
            return Err(ContractError::ZeroTotalPoints);
        }

        let admin_key = symbol_short!("admin");
        let admin: Address = env.storage().instance().get(&admin_key).expect("not initialized");
        admin.require_auth();

        let escrow_key = symbol_short!("escrow");
        let escrow_address: Address = env.storage().instance().get(&escrow_key).expect("not initialized");

        let contributor_count = results.len();

        for item in results.iter() {
            let contributor = item.address;
            let points = item.points;
            
            // reward = (points * wave_reward_pool) / total_points
            let reward = points
                .checked_mul(wave_reward_pool)
                .ok_or(ContractError::ZeroTotalPoints)? // Using ZeroTotalPoints as a fallback error if overflow occurs
                .checked_div(total_points)
                .ok_or(ContractError::ZeroTotalPoints)?;
            
            if reward > 0 {
                env.invoke_contract::<()>(
                    &escrow_address,
                    &symbol_short!("release"),
                    (wave_id.clone(), contributor, reward).into_val(&env),
                );
            }
        }

        // Mark wave as Settled
        env.storage().instance().set(&wave_status_key, &WaveStatus::Settled);

        // Emit Settled(wave_id, total_points, contributor_count)
        env.events().publish(
            (symbol_short!("Settled"), wave_id),
            (total_points, contributor_count),
        );

        Ok(())
    }

    /// Check if a wave is settled
    pub fn is_wave_settled(env: Env, wave_id: String) -> bool {
        let wave_status_key = (symbol_short!("status"), wave_id);
        let wave_status: WaveStatus = env
            .storage()
            .instance()
            .get(&wave_status_key)
            .unwrap_or(WaveStatus::Pending);
        
        wave_status == WaveStatus::Settled
    }

    /// Get registry contract address
    pub fn get_registry_contract(env: Env) -> Address {
        let registry_key = symbol_short!("registry");
        env.storage().instance().get(&registry_key).expect("not initialized")
    }

    /// Get settlement count
    pub fn get_settlement_count(env: Env) -> u32 {
        let count_key = symbol_short!("stl_cnt");
        env.storage().instance().get::<_, u32>(&count_key).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
