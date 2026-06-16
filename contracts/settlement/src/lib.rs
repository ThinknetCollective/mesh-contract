#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Vec};

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
