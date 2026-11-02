use soroban_sdk::{contracttype, Address, String, Symbol};

pub const PERSISTENT_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 172800; // ~10 days

/// Roles of agricultural ecosystem participants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ParticipantRole {
    Farmer = 1,
    Buyer = 2,
    Arbiter = 3,
    Inspector = 4,
}

/// Participant profile with role and IPFS/metadata URI.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticipantProfile {
    pub address: Address,
    pub role: ParticipantRole,
    pub metadata_uri: String,
    pub is_active: bool,
    pub registered_at: u64,
}

/// Standardized agricultural commodity specification.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CommoditySpec {
    pub commodity_id: Symbol,
    pub base_moisture_bps: u32,
    pub max_moisture_bps: u32,
    pub max_tolerance_bps: u32,
    pub shrinkage_factor_bps: u32,
    pub is_active: bool,
}

/// Storage keys for the Registry contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Participant(Address),
    ParticipantCount,
    Commodity(Symbol),
    TokenWhitelist(Address),
}
