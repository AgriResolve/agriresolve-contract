use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeInitialized {
    #[topic]
    pub trade_id: u64,
    pub buyer: Address,
    pub farmer: Address,
    pub token: Address,
    pub amount: i128,
    pub expiry: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryConfirmed {
    #[topic]
    pub trade_id: u64,
    pub farmer: Address,
    pub delivery_hash: BytesN<32>,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeFlagged {
    #[topic]
    pub trade_id: u64,
    pub caller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionExecuted {
    #[topic]
    pub trade_id: u64,
    pub arbiter: Address,
    pub buyer_share: i128,
    pub farmer_share: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeRefunded {
    #[topic]
    pub trade_id: u64,
    pub recipient: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeContractConfigured {
    #[topic]
    pub dispute_contract: Address,
}

pub fn emit_trade_initialized(
    env: &Env,
    trade_id: u64,
    buyer: &Address,
    farmer: &Address,
    token: &Address,
    amount: i128,
    expiry: u64,
) {
    TradeInitialized {
        trade_id,
        buyer: buyer.clone(),
        farmer: farmer.clone(),
        token: token.clone(),
        amount,
        expiry,
    }
    .publish(env);
}

pub fn emit_delivery_confirmed(
    env: &Env,
    trade_id: u64,
    farmer: &Address,
    delivery_hash: BytesN<32>,
) {
    DeliveryConfirmed {
        trade_id,
        farmer: farmer.clone(),
        delivery_hash,
    }
    .publish(env);
}

pub fn emit_dispute_flagged(env: &Env, trade_id: u64, caller: &Address) {
    DisputeFlagged {
        trade_id,
        caller: caller.clone(),
    }
    .publish(env);
}

pub fn emit_resolution_executed(
    env: &Env,
    trade_id: u64,
    arbiter: &Address,
    buyer_share: i128,
    farmer_share: i128,
) {
    ResolutionExecuted {
        trade_id,
        arbiter: arbiter.clone(),
        buyer_share,
        farmer_share,
    }
    .publish(env);
}

pub fn emit_trade_refunded(env: &Env, trade_id: u64, recipient: &Address, amount: i128) {
    TradeRefunded {
        trade_id,
        recipient: recipient.clone(),
        amount,
    }
    .publish(env);
}

pub fn emit_dispute_contract_configured(env: &Env, dispute_contract: &Address) {
    DisputeContractConfigured {
        dispute_contract: dispute_contract.clone(),
    }
    .publish(env);
}
