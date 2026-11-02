#![cfg(test)]

use crate::{
    errors::ContractError, storage::ParticipantRole, RegistryContract, RegistryContractClient,
};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

struct TestFixture<'a> {
    env: Env,
    client: RegistryContractClient<'a>,
    admin: Address,
}

impl<'a> TestFixture<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register(RegistryContract, ());
        let client = RegistryContractClient::new(&env, &contract_id);

        client.initialize(&admin);

        Self { env, client, admin }
    }
}

#[test]
fn test_initialize_and_admin_transfer() {
    let fix = TestFixture::setup();

    assert_eq!(fix.client.get_admin(), fix.admin);

    // Re-initialize rejected
    let res = fix.client.try_initialize(&fix.admin);
    assert_eq!(res, Err(Ok(ContractError::AlreadyInitialized)));

    // Admin transfer
    let new_admin = Address::generate(&fix.env);
    fix.client.set_admin(&fix.admin, &new_admin);
    assert_eq!(fix.client.get_admin(), new_admin);

    // Old admin unauthorized
    let res_unauth = fix.client.try_set_admin(&fix.admin, &fix.admin);
    assert_eq!(res_unauth, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_register_participant_happy_path() {
    let fix = TestFixture::setup();
    let farmer = Address::generate(&fix.env);
    let meta = String::from_str(&fix.env, "ipfs://bafybeifarm123");

    fix.client
        .register_participant(&farmer, &farmer, &ParticipantRole::Farmer, &meta);

    assert_eq!(fix.client.get_participant_count(), 1);
    let profile = fix.client.get_participant(&farmer);
    assert_eq!(profile.address, farmer);
    assert_eq!(profile.role, ParticipantRole::Farmer);
    assert_eq!(profile.metadata_uri, meta);
    assert!(profile.is_active);

    assert!(fix.client.is_participant_active(&farmer));
    assert!(fix
        .client
        .is_participant_role(&farmer, &ParticipantRole::Farmer));
    assert!(!fix
        .client
        .is_participant_role(&farmer, &ParticipantRole::Buyer));
}

#[test]
fn test_participant_status_update() {
    let fix = TestFixture::setup();
    let buyer = Address::generate(&fix.env);
    let meta = String::from_str(&fix.env, "ipfs://buyer-kyc-hash");

    fix.client
        .register_participant(&fix.admin, &buyer, &ParticipantRole::Buyer, &meta);

    assert!(fix.client.is_participant_active(&buyer));

    // Admin deactivates
    fix.client
        .update_participant_status(&fix.admin, &buyer, &false);
    assert!(!fix.client.is_participant_active(&buyer));

    // Non-admin unauthorized to update status
    let stranger = Address::generate(&fix.env);
    let res = fix
        .client
        .try_update_participant_status(&stranger, &buyer, &true);
    assert_eq!(res, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_register_commodity_spec() {
    let fix = TestFixture::setup();
    let maize = symbol_short!("MAIZE");

    // 12.00% base moisture, 15.00% max moisture, 5.00% tolerance, 1.25% shrinkage per point
    fix.client
        .register_commodity(&fix.admin, &maize, &1200, &1500, &500, &125);

    let spec = fix.client.get_commodity(&maize);
    assert_eq!(spec.commodity_id, maize);
    assert_eq!(spec.base_moisture_bps, 1200);
    assert_eq!(spec.max_moisture_bps, 1500);
    assert_eq!(spec.max_tolerance_bps, 500);
    assert_eq!(spec.shrinkage_factor_bps, 125);
    assert!(spec.is_active);
}

#[test]
fn test_invalid_commodity_spec() {
    let fix = TestFixture::setup();
    let soy = symbol_short!("SOY");

    // base > max
    let res = fix
        .client
        .try_register_commodity(&fix.admin, &soy, &1600, &1400, &500, &100);
    assert_eq!(res, Err(Ok(ContractError::InvalidSpecification)));
}

#[test]
fn test_token_whitelist() {
    let fix = TestFixture::setup();
    let usdc = Address::generate(&fix.env);

    assert!(!fix.client.is_token_approved(&usdc));

    fix.client.set_token_whitelist(&fix.admin, &usdc, &true);
    assert!(fix.client.is_token_approved(&usdc));

    fix.client.set_token_whitelist(&fix.admin, &usdc, &false);
    assert!(!fix.client.is_token_approved(&usdc));
}
