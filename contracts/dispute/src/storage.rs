use soroban_sdk::{contracttype, Address, BytesN, Symbol};

pub const PERSISTENT_BUMP_AMOUNT: u32 = 518400; // ~30 days at 5s/ledger
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 172800; // ~10 days

/// Status of an agricultural trade dispute.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DisputeStatus {
    Pending = 0,
    UnderReview = 1,
    Resolved = 2,
    Dismissed = 3,
}

/// Cryptographic evidence submission record.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceRecord {
    pub evidence_type: Symbol,
    pub evidence_hash: BytesN<32>,
    pub submitter: Address,
    pub submitted_at: u64,
}

/// On-chain dispute dossier linking escrow and resolution proofs.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DisputeRecord {
    pub dispute_id: u64,
    pub escrow_id: u64,
    pub claimant: Address,
    pub complaint_hash: BytesN<32>,
    pub arbiter: Address,
    pub status: DisputeStatus,
    pub evidence_count: u32,
    pub buyer_share: i128,
    pub farmer_share: i128,
    pub resolution_hash: Option<BytesN<32>>,
    pub created_at: u64,
    pub resolved_at: u64,
}

/// Storage keys for the Dispute contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    EscrowContract,
    DisputeCount,
    Dispute(u64),
    DisputeByEscrow(u64),
    Evidence(u64, u32),
}
