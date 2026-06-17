#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow with registry address and admin
    pub fn init(env: Env, registry: Address, admin: Address) {
        let registry_key = symbol_short!("registry");
        let admin_key = symbol_short!("admin");
        let count_key = symbol_short!("wave_cnt");
        env.storage().instance().set(&registry_key, &registry);
        env.storage().instance().set(&admin_key, &admin);
        env.storage().instance().set(&count_key, &0u32);
    }

    /// Open a new Wave escrow
    pub fn open_wave(
        env: Env,
        wave_id: String,
        program_id: String,
        creator: Address,
        amount: i128,
    ) {
        let registry_key = symbol_short!("registry");
        let _registry: Address = env.storage().instance().get::<_, Address>(&registry_key).unwrap();
        
        // Verify program exists in registry
        // In production, you'd call the registry contract here
        
        let wave_key = symbol_short!("wave");
        let wave_data: (String, Address, i128, u32) = (
            program_id,
            creator,
            amount,
            0u32, // Open status
        );

        env.storage().instance().set(&(wave_key, wave_id.clone()), &wave_data);

        let count_key = symbol_short!("wave_cnt");
        let mut count: u32 = env.storage().instance().get::<_, u32>(&count_key).unwrap();
        count += 1;
        env.storage().instance().set(&count_key, &count);
    }

    /// Fund a Wave escrow
    pub fn fund_wave(env: Env, wave_id: String, _funder: Address, amount: i128) {
        let wave_key = symbol_short!("wave");
        if let Some((program_id, creator, current_amount, status)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key, wave_id.clone()))
        {
            if status != 0u32 {
                panic!("Wave is not open for funding");
            }
            
            let new_amount = current_amount + amount;
            let wave_key = symbol_short!("wave");
            env.storage()
                .instance()
                .set(&(wave_key, wave_id), &(program_id, creator, new_amount, 1u32)); // Funded status
        } else {
            panic!("Wave not found");
        }
    }

    /// Get Wave details
    pub fn get_wave(env: Env, wave_id: String) -> Option<(String, Address, i128, u32)> {
        let wave_key = symbol_short!("wave");
        env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key, wave_id))
    }

    /// Update Wave status (called by settlement contract)
    pub fn update_status(env: Env, wave_id: String, status: u32) {
        let admin_key = symbol_short!("admin");
        let admin: Address = env.storage().instance().get::<_, Address>(&admin_key).unwrap();
        admin.require_auth();

        let wave_key = symbol_short!("wave");
        if let Some((program_id, creator, amount, _)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key, wave_id.clone()))
        {
            let wave_key = symbol_short!("wave");
            env.storage()
                .instance()
                .set(&(wave_key, wave_id), &(program_id, creator, amount, status));
        }
    }

    /// Release funds from a Wave escrow (called by settlement contract)
    pub fn release(env: Env, wave_id: String, _contributor: Address, amount: i128) {
        let wave_key = symbol_short!("wave");
        if let Some((program_id, creator, current_amount, status)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key, wave_id.clone()))
        {
            // Allow status 1 (Funded) or 2 (Settling/Settled)
            if status != 1u32 && status != 2u32 {
                panic!("Wave is not funded or already finalized");
            }

            if amount > current_amount {
                panic!("Insufficient funds in escrow");
            }

            let new_amount = current_amount - amount;
            env.storage()
                .instance()
                .set(&(wave_key, wave_id), &(program_id, creator, new_amount, status));
        } else {
            panic!("Wave not found");
        }
    }

    /// Get wave count
    pub fn get_wave_count(env: Env) -> u32 {
        let count_key = symbol_short!("wave_cnt");
        env.storage().instance().get::<_, u32>(&count_key).unwrap()
    }
}

#[cfg(test)]
mod test;
