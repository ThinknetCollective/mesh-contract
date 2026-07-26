use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    Unauthorized = 2,
    AlreadyInitialized = 3,
    ProgramNotFound = 4,
    WaveNotFound = 5,
    SettlementNotSet = 6,
    ProgramNameExists = 7,
}
