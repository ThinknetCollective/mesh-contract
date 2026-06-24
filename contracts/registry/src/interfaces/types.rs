use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramConfig {
    pub name: String,
    pub organizer: Address,
    pub metadata: String,
    pub funding_target: u128,
}
