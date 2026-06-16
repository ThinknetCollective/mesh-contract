#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String};

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the registry with admin and escrow contract addresses
    pub fn initialize(env: Env, admin: Address, escrow_contract: Address) {
        if env.storage().instance().has(&String::from_str(&env, "admin")) {
            panic!("already initialized");
        }
        
        env.storage().instance().set(&String::from_str(&env, "admin"), &admin);
        env.storage().instance().set(&String::from_str(&env, "escrow_contract"), &escrow_contract);
    }

    /// Register a new Wave Program
    pub fn register_program(
        env: Env,
        program_id: String,
        creator: Address,
        metadata: String,
        funding_target: u128,
    ) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized");

        admin.require_auth();

        let mut programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        if programs.contains_key(program_id.clone()) {
            panic!("program already exists");
        }

        programs.set(program_id.clone(), (creator, metadata, funding_target, false));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "programs"), &programs);
    }

    /// Get program details
    pub fn get_program(env: Env, program_id: String) -> Option<(Address, String, u128, bool)> {
        let programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        programs.get(program_id)
    }

    /// Settle a program (mark as completed)
    pub fn settle_program(env: Env, program_id: String) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized");

        admin.require_auth();

        let mut programs: Map<String, (Address, String, u128, bool)> = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "programs"))
            .unwrap_or(Map::new(&env));

        let program = programs
            .get(program_id.clone())
            .expect("program not found");

        programs.set(program_id.clone(), (program.0, program.1, program.2, true));
        env.storage()
            .instance()
            .set(&String::from_str(&env, "programs"), &programs);
    }

    /// Get escrow contract address
    pub fn get_escrow_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "escrow_contract"))
            .expect("not initialized")
    }
}
