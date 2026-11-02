use soroban_sdk::{contractevent, Address, Env, String, Symbol};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantRegistered {
    #[topic]
    pub participant: Address,
    pub role: u32,
    pub metadata_uri: String,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantStatusUpdated {
    #[topic]
    pub participant: Address,
    pub is_active: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommoditySpecRegistered {
    #[topic]
    pub commodity_id: Symbol,
    pub base_moisture_bps: u32,
    pub max_moisture_bps: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenWhitelistUpdated {
    #[topic]
    pub token: Address,
    pub is_approved: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferred {
    #[topic]
    pub old_admin: Address,
    pub new_admin: Address,
}

pub fn emit_participant_registered(
    env: &Env,
    participant: &Address,
    role: u32,
    metadata_uri: &String,
) {
    ParticipantRegistered {
        participant: participant.clone(),
        role,
        metadata_uri: metadata_uri.clone(),
    }
    .publish(env);
}

pub fn emit_participant_status_updated(env: &Env, participant: &Address, is_active: bool) {
    ParticipantStatusUpdated {
        participant: participant.clone(),
        is_active,
    }
    .publish(env);
}

pub fn emit_commodity_registered(
    env: &Env,
    commodity_id: &Symbol,
    base_moisture_bps: u32,
    max_moisture_bps: u32,
) {
    CommoditySpecRegistered {
        commodity_id: commodity_id.clone(),
        base_moisture_bps,
        max_moisture_bps,
    }
    .publish(env);
}

pub fn emit_token_whitelist_updated(env: &Env, token: &Address, is_approved: bool) {
    TokenWhitelistUpdated {
        token: token.clone(),
        is_approved,
    }
    .publish(env);
}

pub fn emit_admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    AdminTransferred {
        old_admin: old_admin.clone(),
        new_admin: new_admin.clone(),
    }
    .publish(env);
}
