#![no_std]

pub mod errors;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;

use errors::ContractError;
use events::{
    emit_admin_transferred, emit_commodity_registered, emit_participant_registered,
    emit_participant_status_updated, emit_token_whitelist_updated,
};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};
use storage::{
    CommoditySpec, DataKey, ParticipantProfile, ParticipantRole, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_LIFETIME_THRESHOLD,
};

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// Initialize the registry contract with protocol admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        Ok(())
    }

    /// Transfer administrative control to a new address.
    pub fn set_admin(env: Env, caller: Address, new_admin: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let current_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != current_admin {
            return Err(ContractError::Unauthorized);
        }

        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_admin_transferred(&env, &current_admin, &new_admin);

        Ok(())
    }

    /// Read the protocol administrator address.
    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    /// Register or update an ecosystem participant profile.
    pub fn register_participant(
        env: Env,
        caller: Address,
        participant: Address,
        role: ParticipantRole,
        metadata_uri: String,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        // Only admin or participant self-registering is authorized
        if caller != admin && caller != participant {
            return Err(ContractError::Unauthorized);
        }

        let p_key = DataKey::Participant(participant.clone());
        let is_new = !env.storage().persistent().has(&p_key);

        let profile = ParticipantProfile {
            address: participant.clone(),
            role,
            metadata_uri: metadata_uri.clone(),
            is_active: true,
            registered_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&p_key, &profile);
        env.storage().persistent().extend_ttl(
            &p_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        if is_new {
            let mut count: u64 = env
                .storage()
                .persistent()
                .get(&DataKey::ParticipantCount)
                .unwrap_or(0);
            count = count.checked_add(1).ok_or(ContractError::Overflow)?;
            env.storage()
                .persistent()
                .set(&DataKey::ParticipantCount, &count);
            env.storage().persistent().extend_ttl(
                &DataKey::ParticipantCount,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
        }

        emit_participant_registered(&env, &participant, role as u32, &metadata_uri);

        Ok(())
    }

    /// Suspend or reactivate a participant profile (admin only).
    pub fn update_participant_status(
        env: Env,
        caller: Address,
        participant: Address,
        is_active: bool,
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

        let p_key = DataKey::Participant(participant.clone());
        let mut profile: ParticipantProfile = env
            .storage()
            .persistent()
            .get(&p_key)
            .ok_or(ContractError::ParticipantNotFound)?;

        profile.is_active = is_active;
        env.storage().persistent().set(&p_key, &profile);
        env.storage().persistent().extend_ttl(
            &p_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_participant_status_updated(&env, &participant, is_active);

        Ok(())
    }

    /// Read participant profile details.
    pub fn get_participant(
        env: Env,
        participant: Address,
    ) -> Result<ParticipantProfile, ContractError> {
        let p_key = DataKey::Participant(participant);
        env.storage()
            .persistent()
            .get(&p_key)
            .ok_or(ContractError::ParticipantNotFound)
    }

    /// Check if participant is active.
    pub fn is_participant_active(env: Env, participant: Address) -> bool {
        let p_key = DataKey::Participant(participant);
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<DataKey, ParticipantProfile>(&p_key)
        {
            profile.is_active
        } else {
            false
        }
    }

    /// Check if participant has a specific role and is active.
    pub fn is_participant_role(env: Env, participant: Address, role: ParticipantRole) -> bool {
        let p_key = DataKey::Participant(participant);
        if let Some(profile) = env
            .storage()
            .persistent()
            .get::<DataKey, ParticipantProfile>(&p_key)
        {
            profile.is_active && profile.role == role
        } else {
            false
        }
    }

    /// Total registered participants count.
    pub fn get_participant_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::ParticipantCount)
            .unwrap_or(0)
    }

    /// Register or update standard commodity specifications (admin only).
    pub fn register_commodity(
        env: Env,
        caller: Address,
        commodity_id: Symbol,
        base_moisture_bps: u32,
        max_moisture_bps: u32,
        max_tolerance_bps: u32,
        shrinkage_factor_bps: u32,
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

        if base_moisture_bps > max_moisture_bps || max_moisture_bps > 10000 {
            return Err(ContractError::InvalidSpecification);
        }

        let spec = CommoditySpec {
            commodity_id: commodity_id.clone(),
            base_moisture_bps,
            max_moisture_bps,
            max_tolerance_bps,
            shrinkage_factor_bps,
            is_active: true,
        };

        let c_key = DataKey::Commodity(commodity_id.clone());
        env.storage().persistent().set(&c_key, &spec);
        env.storage().persistent().extend_ttl(
            &c_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_commodity_registered(&env, &commodity_id, base_moisture_bps, max_moisture_bps);

        Ok(())
    }

    /// Retrieve commodity quality and moisture specifications.
    pub fn get_commodity(env: Env, commodity_id: Symbol) -> Result<CommoditySpec, ContractError> {
        let c_key = DataKey::Commodity(commodity_id);
        env.storage()
            .persistent()
            .get(&c_key)
            .ok_or(ContractError::CommodityNotFound)
    }

    /// Whitelist or revoke settlement token address (admin only).
    pub fn set_token_whitelist(
        env: Env,
        caller: Address,
        token: Address,
        is_approved: bool,
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

        let t_key = DataKey::TokenWhitelist(token.clone());
        env.storage().persistent().set(&t_key, &is_approved);
        env.storage().persistent().extend_ttl(
            &t_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_token_whitelist_updated(&env, &token, is_approved);

        Ok(())
    }

    /// Check if token address is approved for settlement escrow.
    pub fn is_token_approved(env: Env, token: Address) -> bool {
        let t_key = DataKey::TokenWhitelist(token);
        env.storage().persistent().get(&t_key).unwrap_or(false)
    }
}
