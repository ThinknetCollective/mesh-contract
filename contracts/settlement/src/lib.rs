#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Vec};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};

#[contract]
pub struct SettlementContract;

#[contractimpl]
impl SettlementContract {
    /// Initialize the settlement with admin, registry, and escrow contract addresses
    pub fn initialize(
        env: Env,
        admin: Address,
        registry_contract: Address,
        escrow_contract: Address,
    ) {
        if env.storage().instance().has(&String::from_str(&env, "admin")) {
            panic!("already initialized");
        }

        env.storage().instance().set(&String::from_str(&env, "admin"), &admin);
        env.storage()
            .instance()
            .set(&String::from_str(&env, "registry_contract"), &registry_contract);
        env.storage()
            .instance()
            .set(&String::from_str(&env, "escrow_contract"), &escrow_contract);
    }

    /// Settle a wave - release funds to recipient
    pub fn settle_wave(env: Env, wave_id: String, recipient: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized");

        admin.require_auth();

        let escrow_contract: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "escrow_contract"))
            .expect("not initialized");

        // Call escrow to release funds
        // Note: Cross-contract calls require proper client setup
        // For now, we'll mark the wave as settled
        let mut settled_waves: Map<String, Address> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "settled_waves"))
            .unwrap_or(Map::new(&env));

        settled_waves.set(wave_id, recipient);
        env.storage()
            .instance()
            .set(&String::from_str(&env, "settled_waves"), &settled_waves);
    }

    /// Check if a wave is settled
    pub fn is_wave_settled(env: Env, wave_id: String) -> bool {
        let settled_waves: Map<String, Address> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "settled_waves"))
            .unwrap_or(Map::new(&env));

        settled_waves.contains_key(wave_id)
    }

    /// Get registry contract address
    pub fn get_registry_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "registry_contract"))
            .expect("not initialized")
    }
}
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
}

#[cfg(test)]
mod test;
