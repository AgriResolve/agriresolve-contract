use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    DisputeNotFound = 4,
    DisputeAlreadyExists = 5,
    InvalidDisputeState = 6,
    EscrowNotFound = 7,
    InvalidSplit = 8,
    EvidenceLimitExceeded = 9,
    ZeroAddress = 10,
    EscrowCallFailed = 11,
    Overflow = 12,
}
