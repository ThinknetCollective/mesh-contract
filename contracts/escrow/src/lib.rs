#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Vec};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String};

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
