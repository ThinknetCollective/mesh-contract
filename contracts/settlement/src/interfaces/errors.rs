use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    EmptyResults = 1,
    ZeroTotalPoints = 2,
    DuplicateContributor = 3,
    AlreadySettled = 4,
}
