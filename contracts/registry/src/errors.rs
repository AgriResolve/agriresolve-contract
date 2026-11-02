use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ParticipantNotFound = 4,
    ParticipantAlreadyExists = 5,
    ParticipantInactive = 6,
    CommodityNotFound = 7,
    CommodityAlreadyExists = 8,
    InvalidSpecification = 9,
    TokenNotWhitelisted = 10,
    ZeroAddress = 11,
    Overflow = 12,
}
