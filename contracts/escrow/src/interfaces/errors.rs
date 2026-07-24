use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    WaveAlreadyExists = 3,
    WaveNotOpen = 4,
    WaveNotFound = 5,
    Unauthorized = 6,
}
