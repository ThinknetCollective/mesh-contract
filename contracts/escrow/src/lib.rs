#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Vec};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow with admin, registry, and settlement contract addresses
    pub fn initialize(
        env: Env,
        admin: Address,
        registry_contract: Address,
        settlement_contract: Address,
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
            .set(&String::from_str(&env, "settlement_contract"), &settlement_contract);
    }

    /// Open a new Wave (escrow) for a program
    pub fn open_wave(
        env: Env,
        wave_id: String,
        program_id: String,
        funder: Address,
        amount: u128,
    ) {
        funder.require_auth();

        let mut waves: Map<String, (String, Address, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "waves"))
            .unwrap_or(Map::new(&env));

        if waves.contains_key(wave_id.clone()) {
            panic!("wave already exists");
        }

        waves.set(wave_id.clone(), (program_id, funder, amount, false));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "waves"), &waves);
    }

    /// Fund an existing wave
    pub fn fund_wave(env: Env, wave_id: String, funder: Address, amount: u128) {
        funder.require_auth();

        let mut waves: Map<String, (String, Address, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "waves"))
            .unwrap_or(Map::new(&env));

        let wave = waves.get(wave_id.clone()).expect("wave not found");
        waves.set(
            wave_id.clone(),
            (wave.0, wave.1, wave.2 + amount, wave.3),
        );
        env.storage()
            .instance()
            .set(&String::from_str(&env, "waves"), &waves);
    }

    /// Get wave details
    pub fn get_wave(env: Env, wave_id: String) -> Option<(String, Address, u128, bool)> {
        let waves: Map<String, (String, Address, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "waves"))
            .unwrap_or(Map::new(&env));

        waves.get(wave_id)
    }

    /// Release funds from a wave (called by settlement contract)
    pub fn release_funds(env: Env, wave_id: String, recipient: Address) {
        let settlement_contract: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "settlement_contract"))
            .expect("not initialized");

        settlement_contract.require_auth();

        let mut waves: Map<String, (String, Address, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "waves"))
            .unwrap_or(Map::new(&env));

        let wave = waves.get(wave_id.clone()).expect("wave not found");
        waves.set(wave_id.clone(), (wave.0, wave.1, wave.2, true));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "waves"), &waves);
    }

    /// Get registry contract address
    pub fn get_registry_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "registry_contract"))
            .expect("not initialized")
    }
}
