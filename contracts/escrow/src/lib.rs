#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, String};
mod interfaces;

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
        if env.storage().instance().has(&symbol_short!("admin")) {
            panic!("already initialized");
        }

        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&String::from_str(&env, "registry_contract"), &registry_contract);
        env.storage()
            .instance()
            .set(&String::from_str(&env, "settlement_contract"), &settlement_contract);
        env.storage().instance().set(&symbol_short!("wave_cnt"), &0u32);
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .expect("not initialized")
    }

    /// Get registry contract address
    pub fn get_registry_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "registry_contract"))
            .expect("not initialized")
    }

    /// Get settlement contract address
    pub fn get_settlement_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "settlement_contract"))
            .expect("not initialized")
    }
    /// Open a new Wave escrow
    pub fn open_wave(
        env: Env,
        wave_id: String,
        program_id: String,
        creator: Address,
        amount: i128,
    ) {
        let wave_key = symbol_short!("wave");
        
        // Check if wave already exists
        if env.storage().instance().has(&(wave_key.clone(), wave_id.clone())) {
            panic!("wave already exists");
        }
        
        let wave_data: (String, Address, i128, u32) = (
            program_id,
            creator,
            amount,
            0u32, // Open status
        );

        env.storage().instance().set(&(wave_key, wave_id.clone()), &wave_data);

        let count_key = symbol_short!("wave_cnt");
        let mut count: u32 = env.storage().instance().get(&count_key).unwrap_or(0u32);
        count += 1;
        env.storage().instance().set(&count_key, &count);
    }

    /// Fund a Wave escrow
    pub fn fund_wave(env: Env, wave_id: String, funder: Address, amount: i128) {
        funder.require_auth();
        
        let wave_key = symbol_short!("wave");
        if let Some((program_id, creator, current_amount, status)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key.clone(), wave_id.clone()))
        {
            if status != 0u32 {
                panic!("Wave is not open for funding");
            }
            
            let new_amount = current_amount + amount;
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

    /// Get wave count
    pub fn get_wave_count(env: Env) -> u32 {
        let count_key = symbol_short!("wave_cnt");
        env.storage().instance().get(&count_key).unwrap_or(0u32)
    }

    /// Fund a program vault with tokens
    /// 
    /// # Arguments
    /// * `program_id` - The unique identifier of the Wave program
    /// * `token` - The token contract address to deposit
    /// * `amount` - The amount of tokens to deposit
    /// 
    /// # Reverts
    /// * If caller is not authorized (not the program organizer)
    /// * If amount is zero
    /// * If token transfer fails
    /// 
    /// # Emits
    /// * Funded(program_id, amount) event on success
    pub fn fund(env: Env, program_id: u64, token: Address, amount: i128) {
        // Validate amount
        if amount <= 0 {
            panic!("Amount must be greater than zero");
        }

        // Authorization check: caller must be the program organizer
        // For now, we'll use the admin as the authorized funder
        // In production, this should check against the program's organizer
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .expect("not initialized");
        admin.require_auth();

        // Get or create program vault storage
        let balances_key = symbol_short!("program_balances");
        let mut balances: Map<u64, (Address, i128)> = env
            .storage()
            .instance()
            .get(&balances_key)
            .unwrap_or_else(|| Map::new(&env));

        // Get current balance for this program
        let current_balance = balances.get(program_id).unwrap_or((token.clone(), 0i128));

        // Verify token matches if program already has funds
        if current_balance.0 != token && current_balance.1 > 0 {
            panic!("Token mismatch: program already funded with different token");
        }

        // Update balance
        let new_balance = current_balance.1 + amount;
        balances.set(program_id, (token.clone(), new_balance));

        // Store updated balances
        env.storage().instance().set(&balances_key, &balances);

        // TODO: Implement SEP-41 token transfer from caller to escrow
        // This requires calling the token contract's transfer function
        // For now, we'll skip the actual transfer and just track the balance

        // Emit Funded event
        env.events().publish(
            (symbol_short!("Funded"), program_id),
            (token, amount),
        );
    }

    /// Get program vault balance
    pub fn get_program_balance(env: Env, program_id: u64) -> Option<(Address, i128)> {
        let balances_key = symbol_short!("program_balances");
        let balances: Map<u64, (Address, i128)> = env
            .storage()
            .instance()
            .get(&balances_key)
            .unwrap_or_else(|| Map::new(&env));
        balances.get(program_id)
    }

    /// Release funds from a Wave escrow to a recipient
    /// Called by the settlement contract after a Wave cycle closes
    /// 
    /// # Arguments
    /// * `wave_id` - The identifier of the Wave
    /// * `recipient` - The address to receive the funds
    /// * `amount` - The amount to release
    /// 
    /// # Reverts
    /// * If caller is not the authorized settlement contract
    /// * If wave does not exist
    /// * If escrow balance is insufficient
    pub fn release(env: Env, wave_id: String, recipient: Address, amount: i128) {
        // Authorization check: only settlement contract can call this
        let settlement_contract: Address = env
            .storage()
            .instance()
            .get(&String::from_str(&env, "settlement_contract"))
            .expect("not initialized");
        settlement_contract.require_auth();

        let wave_key = symbol_short!("wave");
        if let Some((program_id, creator, current_amount, status)) =
            env.storage().instance().get::<_, (String, Address, i128, u32)>(&(wave_key.clone(), wave_id.clone()))
        {
            // Allow status 1 (Funded) or 2 (Settling/Settled)
            if status != 1u32 && status != 2u32 {
                panic!("Wave is not funded or already finalized");
            }

            // Balance check
            if amount > current_amount {
                panic!("Insufficient funds in escrow");
            }

            // Atomic balance decrement
            let new_amount = current_amount - amount;
            env.storage()
                .instance()
                .set(&(wave_key, wave_id.clone()), &(program_id, creator, new_amount, status));

            // Emit Released event
            env.events().publish(
                (symbol_short!("Released"), wave_id),
                (recipient, amount),
            );
        } else {
            panic!("Wave not found");
        }
    }
}

#[cfg(test)]
mod test;
