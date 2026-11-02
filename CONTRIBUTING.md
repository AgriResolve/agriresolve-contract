# Contributing to AgriResolve Contracts

Thank you for your interest in contributing to **AgriResolve Smart Contracts**! This repository hosts the pure Rust Soroban smart contract workspace powering decentralized agricultural escrow, commodity quality registries, and cryptographic dispute settlement on the Stellar network.

---

## Code of Conduct

We are committed to providing a welcoming, inclusive, and harassment-free environment for all contributors. Please adhere to respectful communication across all issues, pull requests, and discussion threads.

---

## Workspace Architecture

The workspace is organized into three single-responsibility, tightly coupled contracts:

1. **`contracts/registry` (`agriresolve-registry`)**:
   - Manages participant profiles (Farmer, Buyer, Arbiter, Inspector) and identity verification.
   - Stores standardized commodity quality specifications (base moisture, allowable tolerance, moisture shrinkage discounts).
   - Enforces settlement token whitelisting.

2. **`contracts/escrow` (`agriresolve-escrow`)**:
   - Implements multi-party collateral custody and atomic deposits via the SEP-41 Soroban Asset Contract (SAC).
   - Manages the deterministic trade state machine (`Funded` $\rightarrow$ `Delivered` $\rightarrow$ `Disputed` $\rightarrow$ `Resolved` / `Refunded`).
   - Executes atomic split payouts with integer balance conservation.

3. **`contracts/dispute` (`agriresolve-dispute`)**:
   - Manages cryptographic evidence dossiers (weighbridge receipts, moisture analysis reports, phytosanitary certificates).
   - Coordinates cross-contract locking with the escrow engine.
   - Enforces arbiter resolution authorization and dispute finality.

---

## Prerequisites & Development Setup

Ensure you have the following installed:

- **Rust:** `1.84+` (stable)
- **Rust Target:** `wasm32v1-none` (`rustup target add wasm32v1-none`)
- **Stellar CLI:** `25.2.0+` (`cargo install --locked stellar-cli`)

Clone the repository:
```bash
git clone https://github.com/AgriResolve/AgriResolve-contract.git
cd AgriResolve-contract
```

---

## Build, Test & Lint Commands

All standard development tasks are managed via the included `Makefile`:

```bash
# Build the entire workspace
make build

# Run comprehensive unit and integration tests (18+ tests)
make test

# Format code according to Rustfmt
make fmt

# Verify code formatting (CI check)
make fmt-check

# Run Clippy linter with strict warning enforcement
make clippy

# Compile optimized WASM binaries for wasm32v1-none
make build-wasm
```

---

## Commit Guidelines

We enforce the **Conventional Commits** specification:

- `feat(escrow): add milestone release entrypoint`
- `fix(registry): correct shrinkage basis point calculation`
- `test(dispute): add property fuzzing for split allocation`
- `docs(readme): update testnet deployment instructions`
- `ci: add automated coverage reporting workflow`

---

## Pull Request Checklist

Before opening a pull request, ensure:
1. `cargo test --workspace` passes with 100% success.
2. `cargo fmt --all -- --check` reports zero formatting diffs.
3. `cargo clippy --all-targets --workspace -- -D warnings` reports zero warnings.
4. `make build-wasm` compiles all three contracts into optimized WASM binaries.
5. New entrypoints or state modifications have corresponding unit tests in `src/test.rs`.
