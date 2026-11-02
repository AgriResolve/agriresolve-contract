# Security Policy

The AgriResolve development team places the highest priority on the security and mathematical correctness of its smart contracts. Since these contracts handle agricultural collateral and trade settlements on the Stellar network, adherence to formal invariants and defense-in-depth is mandatory.

---

## Supported Versions

| Version | Supported | Notes |
|:---:|:---:|:---|
| `v0.1.x` | ✅ | Active development on Stellar Protocol 22+ Testnet |

---

## Reporting a Vulnerability

If you discover a potential security vulnerability or economic exploit in any AgriResolve smart contract, **please do not disclose it publicly via GitHub issues or social channels.**

Please report findings to our core security team:
- **Email:** `security@agriresolve.org` (or `edunhassan1006@gmail.com`)
- **PGP Key:** Available upon request or via keybase/security team.

### Submission Guidelines
Please include:
1. Description of the vulnerability and attack vector.
2. Step-by-step reproduction instructions or a test case using `soroban-sdk::testutils`.
3. Assessment of impact (potential token loss, state lockup, unauthorized state manipulation).
4. Proposed mitigation or patch if available.

### Disclosure Timeline
- **Initial Response:** Within 24 hours.
- **Triage & Assessment:** Within 72 hours.
- **Patch Release:** As rapid as verification permits, coordinated with affected parties.

---

## Fundamental Smart Contract Invariants

Every pull request and modification must preserve these non-negotiable invariants:

### 1. Conservation of Financial Balances
- The escrow contract **must never retain residual tokens** after executing a resolution or refund:
  $$\text{buyer\_share} + \text{farmer\_share} == \text{trade.amount}$$
- Funds are transferred atomically via the SEP-41 Soroban Asset Contract (SAC).
- No tokens can be withdrawn without an explicit terminal transition (`Resolved` or `Refunded`).

### 2. Strict Authentication & Role Isolation
- All state modifications require explicit `require_auth()` verification:
  - Trade creation requires `buyer.require_auth()`.
  - Delivery confirmation requires `farmer.require_auth()`.
  - Dispute flagging requires `buyer`, `farmer`, or authorized `dispute_contract`.
  - Settlement execution requires assigned `arbiter` or authorized `dispute_contract`.
  - Administrative operations require `admin.require_auth()`.

### 3. Finite State Machine Monotonicity
- Terminal states (`EscrowState::Resolved`, `EscrowState::Refunded`, `DisputeStatus::Resolved`, `DisputeStatus::Dismissed`) are immutable. Once reached, no further transitions or token operations are allowed.
- Delivery confirmation is locked once a dispute is flagged.

### 4. Integer Overflow Protection
- All token arithmetic and sequence increments utilize checked operations (`checked_add`, `checked_sub`, `checked_mul`) and return `ContractError::Overflow` on error.
- Amounts and shares must be non-negative integers (`i128 >= 0`).

### 5. Persistent Storage TTL Management
- Every entry in persistent storage is automatically extended on access:
  - `PERSISTENT_LIFETIME_THRESHOLD`: 172,800 ledgers (~10 days)
  - `PERSISTENT_BUMP_AMOUNT`: 518,400 ledgers (~30 days)
- This guarantees active contracts never suffer state eviction while awaiting delivery or arbitration.
