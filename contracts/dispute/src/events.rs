use soroban_sdk::{contractevent, Address, BytesN, Env, Symbol};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeOpened {
    #[topic]
    pub dispute_id: u64,
    pub escrow_id: u64,
    pub claimant: Address,
    pub complaint_hash: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSubmitted {
    #[topic]
    pub dispute_id: u64,
    pub evidence_index: u32,
    pub submitter: Address,
    pub evidence_type: Symbol,
    pub evidence_hash: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolved {
    #[topic]
    pub dispute_id: u64,
    pub escrow_id: u64,
    pub arbiter: Address,
    pub buyer_share: i128,
    pub farmer_share: i128,
    pub resolution_hash: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeDismissed {
    #[topic]
    pub dispute_id: u64,
    pub escrow_id: u64,
    pub arbiter: Address,
}

pub fn emit_dispute_opened(
    env: &Env,
    dispute_id: u64,
    escrow_id: u64,
    claimant: &Address,
    complaint_hash: BytesN<32>,
) {
    DisputeOpened {
        dispute_id,
        escrow_id,
        claimant: claimant.clone(),
        complaint_hash,
    }
    .publish(env);
}

pub fn emit_evidence_submitted(
    env: &Env,
    dispute_id: u64,
    evidence_index: u32,
    submitter: &Address,
    evidence_type: Symbol,
    evidence_hash: BytesN<32>,
) {
    EvidenceSubmitted {
        dispute_id,
        evidence_index,
        submitter: submitter.clone(),
        evidence_type,
        evidence_hash,
    }
    .publish(env);
}

pub fn emit_dispute_resolved(
    env: &Env,
    dispute_id: u64,
    escrow_id: u64,
    arbiter: &Address,
    buyer_share: i128,
    farmer_share: i128,
    resolution_hash: BytesN<32>,
) {
    DisputeResolved {
        dispute_id,
        escrow_id,
        arbiter: arbiter.clone(),
        buyer_share,
        farmer_share,
        resolution_hash,
    }
    .publish(env);
}

pub fn emit_dispute_dismissed(env: &Env, dispute_id: u64, escrow_id: u64, arbiter: &Address) {
    DisputeDismissed {
        dispute_id,
        escrow_id,
        arbiter: arbiter.clone(),
    }
    .publish(env);
}
