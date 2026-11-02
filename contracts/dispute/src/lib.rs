#![no_std]

pub mod errors;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;

use errors::ContractError;
use events::{
    emit_dispute_dismissed, emit_dispute_opened, emit_dispute_resolved, emit_evidence_submitted,
};
use soroban_sdk::{contract, contractclient, contractimpl, Address, BytesN, Env, Symbol};
use storage::{
    DataKey, DisputeRecord, DisputeStatus, EvidenceRecord, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};

#[contractclient(name = "EscrowClient")]
pub trait EscrowInterface {
    fn flag_dispute(env: Env, caller: Address, escrow_id: u64);
    fn execute_resolution(
        env: Env,
        caller: Address,
        escrow_id: u64,
        buyer_share: i128,
        farmer_share: i128,
    );
}

#[contract]
pub struct DisputeContract;

#[contractimpl]
impl DisputeContract {
    /// Initialize dispute engine with protocol admin and linked escrow contract.
    pub fn initialize(
        env: Env,
        admin: Address,
        escrow_contract: Address,
    ) -> Result<(), ContractError> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::EscrowContract, &escrow_contract);

        env.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage().persistent().extend_ttl(
            &DataKey::EscrowContract,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        Ok(())
    }

    /// Update linked escrow contract (admin only).
    pub fn set_escrow_contract(
        env: Env,
        caller: Address,
        escrow_contract: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::EscrowContract, &escrow_contract);
        env.storage().persistent().extend_ttl(
            &DataKey::EscrowContract,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        Ok(())
    }

    /// Read linked escrow contract address.
    pub fn get_escrow_contract(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::EscrowContract)
            .ok_or(ContractError::NotInitialized)
    }

    /// Raise a formal dispute against an active trade escrow.
    pub fn raise_dispute(
        env: Env,
        caller: Address,
        escrow_id: u64,
        arbiter: Address,
        complaint_hash: BytesN<32>,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let by_escrow_key = DataKey::DisputeByEscrow(escrow_id);
        if env.storage().persistent().has(&by_escrow_key) {
            return Err(ContractError::DisputeAlreadyExists);
        }

        let escrow_addr: Address = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowContract)
            .ok_or(ContractError::NotInitialized)?;

        // Atomically lock funds on escrow contract via cross-contract call
        let escrow_client = EscrowClient::new(&env, &escrow_addr);
        escrow_client.flag_dispute(&caller, &escrow_id);

        // Increment dispute count
        let mut count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0);
        count = count.checked_add(1).ok_or(ContractError::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::DisputeCount, &count);
        env.storage().persistent().extend_ttl(
            &DataKey::DisputeCount,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let dispute = DisputeRecord {
            dispute_id: count,
            escrow_id,
            claimant: caller.clone(),
            complaint_hash: complaint_hash.clone(),
            arbiter,
            status: DisputeStatus::Pending,
            evidence_count: 0,
            buyer_share: 0,
            farmer_share: 0,
            resolution_hash: None,
            created_at: env.ledger().timestamp(),
            resolved_at: 0,
        };

        let d_key = DataKey::Dispute(count);
        env.storage().persistent().set(&d_key, &dispute);
        env.storage().persistent().set(&by_escrow_key, &count);

        env.storage().persistent().extend_ttl(
            &d_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
        env.storage().persistent().extend_ttl(
            &by_escrow_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_dispute_opened(&env, count, escrow_id, &caller, complaint_hash);

        Ok(count)
    }

    /// Submit cryptographic proof evidence (weighbridge, moisture test, photo proof hash).
    pub fn submit_evidence(
        env: Env,
        caller: Address,
        dispute_id: u64,
        evidence_type: Symbol,
        evidence_hash: BytesN<32>,
    ) -> Result<u32, ContractError> {
        caller.require_auth();

        let d_key = DataKey::Dispute(dispute_id);
        let mut dispute: DisputeRecord = env
            .storage()
            .persistent()
            .get(&d_key)
            .ok_or(ContractError::DisputeNotFound)?;

        if dispute.status != DisputeStatus::Pending && dispute.status != DisputeStatus::UnderReview
        {
            return Err(ContractError::InvalidDisputeState);
        }

        let evidence_index = dispute.evidence_count;
        let e_key = DataKey::Evidence(dispute_id, evidence_index);

        let evidence = EvidenceRecord {
            evidence_type: evidence_type.clone(),
            evidence_hash: evidence_hash.clone(),
            submitter: caller.clone(),
            submitted_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&e_key, &evidence);
        env.storage().persistent().extend_ttl(
            &e_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        dispute.evidence_count = evidence_index
            .checked_add(1)
            .ok_or(ContractError::Overflow)?;
        dispute.status = DisputeStatus::UnderReview;

        env.storage().persistent().set(&d_key, &dispute);
        env.storage().persistent().extend_ttl(
            &d_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_evidence_submitted(
            &env,
            dispute_id,
            evidence_index,
            &caller,
            evidence_type,
            evidence_hash,
        );

        Ok(evidence_index)
    }

    /// Arbiter resolves dispute and triggers atomic settlement on linked escrow contract.
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        dispute_id: u64,
        buyer_share: i128,
        farmer_share: i128,
        resolution_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let d_key = DataKey::Dispute(dispute_id);
        let mut dispute: DisputeRecord = env
            .storage()
            .persistent()
            .get(&d_key)
            .ok_or(ContractError::DisputeNotFound)?;

        if caller != dispute.arbiter {
            return Err(ContractError::Unauthorized);
        }

        if dispute.status != DisputeStatus::Pending && dispute.status != DisputeStatus::UnderReview
        {
            return Err(ContractError::InvalidDisputeState);
        }

        if buyer_share < 0 || farmer_share < 0 {
            return Err(ContractError::InvalidSplit);
        }

        let escrow_addr: Address = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowContract)
            .ok_or(ContractError::NotInitialized)?;

        let escrow_client = EscrowClient::new(&env, &escrow_addr);

        // Atomically disburse split payout through escrow contract
        escrow_client.execute_resolution(
            &env.current_contract_address(),
            &dispute.escrow_id,
            &buyer_share,
            &farmer_share,
        );

        dispute.status = DisputeStatus::Resolved;
        dispute.buyer_share = buyer_share;
        dispute.farmer_share = farmer_share;
        dispute.resolution_hash = Some(resolution_hash.clone());
        dispute.resolved_at = env.ledger().timestamp();

        env.storage().persistent().set(&d_key, &dispute);
        env.storage().persistent().extend_ttl(
            &d_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_dispute_resolved(
            &env,
            dispute_id,
            dispute.escrow_id,
            &caller,
            buyer_share,
            farmer_share,
            resolution_hash,
        );

        Ok(())
    }

    /// Dismiss dispute without settlement adjustments (arbiter only).
    pub fn dismiss_dispute(
        env: Env,
        caller: Address,
        dispute_id: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let d_key = DataKey::Dispute(dispute_id);
        let mut dispute: DisputeRecord = env
            .storage()
            .persistent()
            .get(&d_key)
            .ok_or(ContractError::DisputeNotFound)?;

        if caller != dispute.arbiter {
            return Err(ContractError::Unauthorized);
        }

        if dispute.status != DisputeStatus::Pending && dispute.status != DisputeStatus::UnderReview
        {
            return Err(ContractError::InvalidDisputeState);
        }

        dispute.status = DisputeStatus::Dismissed;
        dispute.resolved_at = env.ledger().timestamp();

        env.storage().persistent().set(&d_key, &dispute);
        env.storage().persistent().extend_ttl(
            &d_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_dispute_dismissed(&env, dispute_id, dispute.escrow_id, &caller);

        Ok(())
    }

    /// Read dispute dossier details.
    pub fn get_dispute(env: Env, dispute_id: u64) -> Result<DisputeRecord, ContractError> {
        let d_key = DataKey::Dispute(dispute_id);
        env.storage()
            .persistent()
            .get(&d_key)
            .ok_or(ContractError::DisputeNotFound)
    }

    /// Read submitted evidence by index.
    pub fn get_evidence(
        env: Env,
        dispute_id: u64,
        index: u32,
    ) -> Result<EvidenceRecord, ContractError> {
        let e_key = DataKey::Evidence(dispute_id, index);
        env.storage()
            .persistent()
            .get(&e_key)
            .ok_or(ContractError::DisputeNotFound)
    }

    /// Look up dispute ID by trade escrow ID.
    pub fn get_dispute_by_escrow(env: Env, escrow_id: u64) -> Result<u64, ContractError> {
        let by_escrow_key = DataKey::DisputeByEscrow(escrow_id);
        env.storage()
            .persistent()
            .get(&by_escrow_key)
            .ok_or(ContractError::DisputeNotFound)
    }

    /// Total disputes raised.
    pub fn get_dispute_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0)
    }
}
