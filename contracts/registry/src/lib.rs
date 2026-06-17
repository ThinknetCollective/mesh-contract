#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the registry with an admin address
    pub fn init(env: Env, admin: Address) {
        let admin_key = symbol_short!("admin");
        let count_key = symbol_short!("prog_cnt");
        env.storage().instance().set(&admin_key, &admin);
        env.storage().instance().set(&count_key, &0u32);
    }

    /// Register a new Wave Program
    pub fn register_program(
        env: Env,
        program_id: String,
        name: String,
        description: String,
        creator: Address,
        escrow_contract: Address,
    ) {
        let admin_key = symbol_short!("admin");
        let admin: Address = env.storage().instance().get::<_, Address>(&admin_key).unwrap();
        admin.require_auth();

        let program_key = symbol_short!("program");
        let program_data: (String, String, Address, Address, u32) = (
            name,
            description,
            creator,
            escrow_contract,
            0u32, // Active status
        );

        env.storage().instance().set(&(program_key, program_id.clone()), &program_data);

        let count_key = symbol_short!("prog_cnt");
        let mut count: u32 = env.storage().instance().get::<_, u32>(&count_key).unwrap();
        count += 1;
        env.storage().instance().set(&count_key, &count);
    }

    /// Get program details
    pub fn get_program(env: Env, program_id: String) -> Option<(String, String, Address, Address, u32)> {
        let program_key = symbol_short!("program");
        env.storage().instance().get::<_, (String, String, Address, Address, u32)>(&(program_key, program_id))
    }

    /// Update program status
    pub fn update_status(env: Env, program_id: String, status: u32) {
        let admin_key = symbol_short!("admin");
        let admin: Address = env.storage().instance().get::<_, Address>(&admin_key).unwrap();
        admin.require_auth();

        let program_key = symbol_short!("program");
        if let Some((name, description, creator, escrow, _)) =
            env.storage().instance().get::<_, (String, String, Address, Address, u32)>(&(program_key, program_id.clone()))
        {
            let program_key = symbol_short!("program");
            env.storage()
                .instance()
                .set(&(program_key, program_id), &(name, description, creator, escrow, status));
        }
    }

    /// Get program count
    pub fn get_program_count(env: Env) -> u32 {
        let count_key = symbol_short!("prog_cnt");
        env.storage().instance().get::<_, u32>(&count_key).unwrap()
    }
}

#[cfg(test)]
mod test;
