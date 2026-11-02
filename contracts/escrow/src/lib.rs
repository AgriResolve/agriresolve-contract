#![no_std]

pub mod errors;
pub mod events;
pub mod storage;

#[cfg(test)]
mod test;

use errors::ContractError;
use events::{
    emit_delivery_confirmed, emit_dispute_contract_configured, emit_dispute_flagged,
    emit_resolution_executed, emit_trade_initialized, emit_trade_refunded,
};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};
use storage::{
    DataKey, EscrowState, TradeDetails, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize escrow admin.
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

    /// Set authorized dispute contract address for cross-contract dispute orchestration.
    pub fn set_dispute_contract(
        env: Env,
        caller: Address,
        dispute_contract: Address,
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
            .set(&DataKey::DisputeContract, &dispute_contract);
        env.storage().persistent().extend_ttl(
            &DataKey::DisputeContract,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_dispute_contract_configured(&env, &dispute_contract);

        Ok(())
    }

    /// Read the authorized dispute contract address.
    pub fn get_dispute_contract(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::DisputeContract)
    }

    /// Initialize a new trade escrow and atomically deposit funds.
    pub fn initialize_trade(
        env: Env,
        buyer: Address,
        farmer: Address,
        arbiter: Address,
        token: Address,
        amount: i128,
        expiry: u64,
    ) -> Result<u64, ContractError> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        if buyer == farmer {
            return Err(ContractError::IdenticalParties);
        }
        if expiry <= env.ledger().timestamp() {
            return Err(ContractError::InvalidExpiry);
        }

        // Execute atomic deposit from buyer into escrow contract
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &contract_address, &amount);

        // Increment trade count
        let mut count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::TradeCount)
            .unwrap_or(0);
        count = count.checked_add(1).ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&DataKey::TradeCount, &count);
        env.storage().persistent().extend_ttl(
            &DataKey::TradeCount,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        let trade = TradeDetails {
            id: count,
            buyer: buyer.clone(),
            farmer: farmer.clone(),
            arbiter,
            token: token.clone(),
            amount,
            state: EscrowState::Funded,
            delivery_hash: None,
            expiry,
            buyer_share: 0,
            farmer_share: 0,
            created_at: env.ledger().timestamp(),
        };

        let trade_key = DataKey::Trade(count);
        env.storage().persistent().set(&trade_key, &trade);
        env.storage().persistent().extend_ttl(
            &trade_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_trade_initialized(&env, count, &buyer, &farmer, &token, amount, expiry);

        Ok(count)
    }

    /// Farmer submits confirmation hash of delivered agricultural goods.
    pub fn confirm_delivery(
        env: Env,
        caller: Address,
        escrow_id: u64,
        delivery_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let trade_key = DataKey::Trade(escrow_id);
        let mut trade: TradeDetails = env
            .storage()
            .persistent()
            .get(&trade_key)
            .ok_or(ContractError::TradeNotFound)?;

        if caller != trade.farmer {
            return Err(ContractError::Unauthorized);
        }

        match trade.state {
            EscrowState::Funded => {}
            EscrowState::Delivered => return Err(ContractError::AlreadyDelivered),
            EscrowState::Disputed => return Err(ContractError::DisputeLocked),
            _ => return Err(ContractError::InvalidState),
        }

        if env.ledger().timestamp() > trade.expiry {
            return Err(ContractError::TradeExpired);
        }

        trade.delivery_hash = Some(delivery_hash.clone());
        trade.state = EscrowState::Delivered;

        env.storage().persistent().set(&trade_key, &trade);
        env.storage().persistent().extend_ttl(
            &trade_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_delivery_confirmed(&env, escrow_id, &caller, delivery_hash);

        Ok(())
    }

    /// Flag a dispute, locking funds. Callable by buyer, farmer, or authorized dispute contract.
    pub fn flag_dispute(env: Env, caller: Address, escrow_id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let trade_key = DataKey::Trade(escrow_id);
        let mut trade: TradeDetails = env
            .storage()
            .persistent()
            .get(&trade_key)
            .ok_or(ContractError::TradeNotFound)?;

        let dispute_contract_opt: Option<Address> =
            env.storage().persistent().get(&DataKey::DisputeContract);
        let is_dispute_contract = match dispute_contract_opt {
            Some(dc) => dc == caller,
            None => false,
        };

        if caller != trade.buyer && caller != trade.farmer && !is_dispute_contract {
            return Err(ContractError::Unauthorized);
        }

        if trade.state != EscrowState::Funded && trade.state != EscrowState::Delivered {
            return Err(ContractError::InvalidState);
        }

        trade.state = EscrowState::Disputed;

        env.storage().persistent().set(&trade_key, &trade);
        env.storage().persistent().extend_ttl(
            &trade_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_dispute_flagged(&env, escrow_id, &caller);

        Ok(())
    }

    /// Execute dispute resolution with split payouts.
    /// Callable by designated arbiter or authorized dispute contract.
    pub fn execute_resolution(
        env: Env,
        caller: Address,
        escrow_id: u64,
        buyer_share: i128,
        farmer_share: i128,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let trade_key = DataKey::Trade(escrow_id);
        let mut trade: TradeDetails = env
            .storage()
            .persistent()
            .get(&trade_key)
            .ok_or(ContractError::TradeNotFound)?;

        let dispute_contract_opt: Option<Address> =
            env.storage().persistent().get(&DataKey::DisputeContract);
        let is_dispute_contract = match dispute_contract_opt {
            Some(dc) => dc == caller,
            None => false,
        };

        if caller != trade.arbiter && !is_dispute_contract {
            return Err(ContractError::Unauthorized);
        }

        if trade.state != EscrowState::Disputed {
            return Err(ContractError::InvalidState);
        }

        if buyer_share < 0 || farmer_share < 0 {
            return Err(ContractError::InvalidAmount);
        }

        let total_split = buyer_share
            .checked_add(farmer_share)
            .ok_or(ContractError::Overflow)?;

        if total_split != trade.amount {
            return Err(ContractError::SplitMismatch);
        }

        let token_client = token::Client::new(&env, &trade.token);
        let contract_address = env.current_contract_address();

        if buyer_share > 0 {
            token_client.transfer(&contract_address, &trade.buyer, &buyer_share);
        }
        if farmer_share > 0 {
            token_client.transfer(&contract_address, &trade.farmer, &farmer_share);
        }

        trade.buyer_share = buyer_share;
        trade.farmer_share = farmer_share;
        trade.state = EscrowState::Resolved;

        env.storage().persistent().set(&trade_key, &trade);
        env.storage().persistent().extend_ttl(
            &trade_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_resolution_executed(&env, escrow_id, &caller, buyer_share, farmer_share);

        Ok(())
    }

    /// Mutual cancellation or expiry refund to buyer before goods are dispatched.
    pub fn refund_trade(env: Env, caller: Address, escrow_id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let trade_key = DataKey::Trade(escrow_id);
        let mut trade: TradeDetails = env
            .storage()
            .persistent()
            .get(&trade_key)
            .ok_or(ContractError::TradeNotFound)?;

        if caller != trade.buyer && caller != trade.farmer {
            return Err(ContractError::Unauthorized);
        }

        if trade.state != EscrowState::Funded {
            return Err(ContractError::InvalidState);
        }

        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &trade.token);
        token_client.transfer(&contract_address, &trade.buyer, &trade.amount);

        trade.state = EscrowState::Refunded;
        trade.buyer_share = trade.amount;
        trade.farmer_share = 0;

        env.storage().persistent().set(&trade_key, &trade);
        env.storage().persistent().extend_ttl(
            &trade_key,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        emit_trade_refunded(&env, escrow_id, &trade.buyer, trade.amount);

        Ok(())
    }

    /// Read details of a specific escrow trade.
    pub fn get_trade(env: Env, escrow_id: u64) -> Result<TradeDetails, ContractError> {
        let trade_key = DataKey::Trade(escrow_id);
        env.storage()
            .persistent()
            .get(&trade_key)
            .ok_or(ContractError::TradeNotFound)
    }

    /// Read the total number of trades registered.
    pub fn get_trade_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::TradeCount)
            .unwrap_or(0)
    }
}
