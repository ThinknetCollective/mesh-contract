#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, IntoVal};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributorPoints {
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

    /// Create a settlement proposal for a Wave
    pub fn create_settlement(
        env: Env,
        settlement_id: String,
        wave_id: String,
        recipient: Address,
        amount: i128,
        _proposer: Address,
    ) {
        let settlement_key = symbol_short!("settle");
        let settlement_data: (String, Address, i128, u32) = (
            wave_id,
            recipient,
            amount,
            0u32, // Pending status
        );

        env.storage()
            .instance()
            .set(&(settlement_key, settlement_id.clone()), &settlement_data);

        let count_key = symbol_short!("stl_cnt");
        let mut count: u32 = env.storage().instance().get::<_, u32>(&count_key).unwrap();
        count += 1;
        env.storage().instance().set(&count_key, &count);
    }

    /// Approve a settlement
    pub fn approve_settlement(env: Env, settlement_id: String) {
        let admin_key = symbol_short!("admin");
        let admin: Address = env.storage().instance().get::<_, Address>(&admin_key).unwrap();
        admin.require_auth();

        let settlement_key = symbol_short!("settle");
        if let Some((wave_id, recipient, amount, _)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(settlement_key, settlement_id.clone()))
        {
            let settlement_key = symbol_short!("settle");
            env.storage().instance().set(
                &(settlement_key, settlement_id),
                &(wave_id, recipient, amount, 1u32), // Approved status
            );
        }
    }

    /// Execute a settlement (transfer funds from escrow)
    pub fn execute_settlement(env: Env, settlement_id: String) {
        let settlement_key = symbol_short!("settle");
        if let Some((wave_id, recipient, amount, status)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(settlement_key, settlement_id.clone()))
        {
            if status != 1u32 {
                panic!("Settlement not approved");
            }

            // Update escrow status
            let escrow_key = symbol_short!("escrow");
            let _escrow: Address = env.storage().instance().get::<_, Address>(&escrow_key).unwrap();
            // In production, you'd call the escrow contract here to update status and transfer funds

            let settlement_key = symbol_short!("settle");
            env.storage().instance().set(
                &(settlement_key, settlement_id),
                &(wave_id, recipient, amount, 3u32), // Completed status
            );
        }
    }

    /// Get settlement details
    pub fn get_settlement(env: Env, settlement_id: String) -> Option<(String, Address, i128, u32)> {
        let settlement_key = symbol_short!("settle");
        env.storage().instance().get::<_, (String, Address, i128, u32)>(&(settlement_key, settlement_id))
    }

    /// Get settlement count
    pub fn get_settlement_count(env: Env) -> u32 {
        let count_key = symbol_short!("stl_cnt");
        env.storage().instance().get::<_, u32>(&count_key).unwrap()
    }

    /// Settle a Wave with proportional rewards
    pub fn settle_wave(
        env: Env,
        wave_id: String,
        contributor_points: soroban_sdk::Vec<ContributorPoints>,
        wave_reward_pool: i128,
    ) {
        let mut total_points: i128 = 0;
        for item in contributor_points.iter() {
            total_points += item.points;
        }

        if total_points == 0 {
            panic!("Total points cannot be zero");
        }

        let escrow_key = symbol_short!("escrow");
        let escrow_address: Address = env.storage().instance().get::<_, Address>(&escrow_key).unwrap();

        let mut sum_rewards: i128 = 0;
        let contributor_count = contributor_points.len();

        for item in contributor_points.iter() {
            let contributor = item.address;
            let points = item.points;
            
            // contributor_reward = (contributor_points / total_points) * wave_reward_pool
            // reward = (points * wave_reward_pool) / total_points
            let reward = points
                .checked_mul(wave_reward_pool)
                .unwrap()
                .checked_div(total_points)
                .unwrap();
            
            if reward > 0 {
                sum_rewards += reward;
                
                // Call escrow.release(wave_id, contributor, reward)
                env.invoke_contract::<()>(
                    &escrow_address,
                    symbol_short!("release"),
                    (wave_id.clone(), contributor, reward).into_val(&env),
                );
            }
        }

        // Emit Settled(wave_id, total_points, contributor_count)
        env.events().publish(
            (symbol_short!("Settled"), wave_id),
            (total_points, contributor_count),
        );
    }
}

#[cfg(test)]
mod test;
