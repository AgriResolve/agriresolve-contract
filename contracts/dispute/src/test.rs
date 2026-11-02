#![cfg(test)]

use crate::{
    errors::ContractError, storage::DisputeStatus, DisputeContract, DisputeContractClient,
};
use agriresolve_escrow::{EscrowContract, EscrowContractClient};
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env,
};

struct TestFixture<'a> {
    env: Env,
    dispute_client: DisputeContractClient<'a>,
    escrow_client: EscrowContractClient<'a>,
    token_client: TokenClient<'a>,
    token_admin_client: StellarAssetClient<'a>,
    token_address: Address,
    buyer: Address,
    farmer: Address,
    arbiter: Address,
}

impl<'a> TestFixture<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let farmer = Address::generate(&env);
        let arbiter = Address::generate(&env);

        let token_admin = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = sac.address();

        let escrow_id = env.register(EscrowContract, ());
        let escrow_client = EscrowContractClient::new(&env, &escrow_id);
        escrow_client.initialize(&admin);

        let dispute_id = env.register(DisputeContract, ());
        let dispute_client = DisputeContractClient::new(&env, &dispute_id);
        dispute_client.initialize(&admin, &escrow_id);

        // Authorize dispute contract on escrow contract
        escrow_client.set_dispute_contract(&admin, &dispute_id);

        let token_client = TokenClient::new(&env, &token_address);
        let token_admin_client = StellarAssetClient::new(&env, &token_address);

        Self {
            env,
            dispute_client,
            escrow_client,
            token_client,
            token_admin_client,
            token_address,
            buyer,
            farmer,
            arbiter,
        }
    }
}

#[test]
fn test_raise_dispute_and_evidence_submission_and_resolution() {
    let fix = TestFixture::setup();
    let total_amount = 10_000_000i128; // 10 USDC
    fix.token_admin_client.mint(&fix.buyer, &total_amount);

    let expiry = fix.env.ledger().timestamp() + 3600;
    let escrow_id = fix.escrow_client.initialize_trade(
        &fix.buyer,
        &fix.farmer,
        &fix.arbiter,
        &fix.token_address,
        &total_amount,
        &expiry,
    );

    // Buyer raises dispute
    let complaint_hash = BytesN::from_array(&fix.env, &[77u8; 32]);
    let dispute_id =
        fix.dispute_client
            .raise_dispute(&fix.buyer, &escrow_id, &fix.arbiter, &complaint_hash);

    assert_eq!(dispute_id, 1);
    assert_eq!(fix.dispute_client.get_dispute_count(), 1);
    assert_eq!(
        fix.dispute_client.get_dispute_by_escrow(&escrow_id),
        dispute_id
    );

    // Escrow state should now be locked in Disputed
    let trade = fix.escrow_client.get_trade(&escrow_id);
    assert_eq!(
        trade.state,
        agriresolve_escrow::storage::EscrowState::Disputed
    );

    // Buyer submits moisture analysis proof
    let evidence_hash = BytesN::from_array(&fix.env, &[88u8; 32]);
    let ev_idx = fix.dispute_client.submit_evidence(
        &fix.buyer,
        &dispute_id,
        &symbol_short!("MOISTURE"),
        &evidence_hash,
    );
    assert_eq!(ev_idx, 0);

    let ev_record = fix.dispute_client.get_evidence(&dispute_id, &ev_idx);
    assert_eq!(ev_record.evidence_type, symbol_short!("MOISTURE"));
    assert_eq!(ev_record.evidence_hash, evidence_hash);
    assert_eq!(ev_record.submitter, fix.buyer);

    // Arbiter executes resolution: 60% buyer, 40% farmer
    let buyer_share = 6_000_000i128;
    let farmer_share = 4_000_000i128;
    let resolution_hash = BytesN::from_array(&fix.env, &[99u8; 32]);

    fix.dispute_client.resolve_dispute(
        &fix.arbiter,
        &dispute_id,
        &buyer_share,
        &farmer_share,
        &resolution_hash,
    );

    let dispute = fix.dispute_client.get_dispute(&dispute_id);
    assert_eq!(dispute.status, DisputeStatus::Resolved);
    assert_eq!(dispute.buyer_share, buyer_share);
    assert_eq!(dispute.farmer_share, farmer_share);
    assert_eq!(dispute.resolution_hash, Some(resolution_hash));

    // Balances settled atomically
    assert_eq!(fix.token_client.balance(&fix.buyer), buyer_share);
    assert_eq!(fix.token_client.balance(&fix.farmer), farmer_share);
}

#[test]
fn test_dispute_unauthorized_parties() {
    let fix = TestFixture::setup();
    let total_amount = 10_000_000i128;
    fix.token_admin_client.mint(&fix.buyer, &total_amount);

    let expiry = fix.env.ledger().timestamp() + 3600;
    let escrow_id = fix.escrow_client.initialize_trade(
        &fix.buyer,
        &fix.farmer,
        &fix.arbiter,
        &fix.token_address,
        &total_amount,
        &expiry,
    );

    // Stranger cannot raise dispute (Escrow contract rejects unauthorized dispute flag)
    let stranger = Address::generate(&fix.env);
    let complaint_hash = BytesN::from_array(&fix.env, &[77u8; 32]);
    let res =
        fix.dispute_client
            .try_raise_dispute(&stranger, &escrow_id, &fix.arbiter, &complaint_hash);
    assert!(res.is_err());
}

#[test]
fn test_duplicate_dispute_prevention() {
    let fix = TestFixture::setup();
    let total_amount = 10_000_000i128;
    fix.token_admin_client.mint(&fix.buyer, &total_amount);

    let expiry = fix.env.ledger().timestamp() + 3600;
    let escrow_id = fix.escrow_client.initialize_trade(
        &fix.buyer,
        &fix.farmer,
        &fix.arbiter,
        &fix.token_address,
        &total_amount,
        &expiry,
    );

    let complaint_hash = BytesN::from_array(&fix.env, &[77u8; 32]);
    fix.dispute_client
        .raise_dispute(&fix.buyer, &escrow_id, &fix.arbiter, &complaint_hash);

    // Raising again should fail with DisputeAlreadyExists
    let res = fix.dispute_client.try_raise_dispute(
        &fix.farmer,
        &escrow_id,
        &fix.arbiter,
        &complaint_hash,
    );
    assert_eq!(res, Err(Ok(ContractError::DisputeAlreadyExists)));
}

#[test]
fn test_dispute_dismissal() {
    let fix = TestFixture::setup();
    let total_amount = 10_000_000i128;
    fix.token_admin_client.mint(&fix.buyer, &total_amount);

    let expiry = fix.env.ledger().timestamp() + 3600;
    let escrow_id = fix.escrow_client.initialize_trade(
        &fix.buyer,
        &fix.farmer,
        &fix.arbiter,
        &fix.token_address,
        &total_amount,
        &expiry,
    );

    let complaint_hash = BytesN::from_array(&fix.env, &[77u8; 32]);
    let dispute_id =
        fix.dispute_client
            .raise_dispute(&fix.buyer, &escrow_id, &fix.arbiter, &complaint_hash);

    // Arbiter dismisses frivolous dispute
    fix.dispute_client
        .dismiss_dispute(&fix.arbiter, &dispute_id);

    let dispute = fix.dispute_client.get_dispute(&dispute_id);
    assert_eq!(dispute.status, DisputeStatus::Dismissed);
}
