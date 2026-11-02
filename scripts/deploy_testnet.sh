#!/usr/bin/env bash
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
SOURCE_ACCOUNT="${STELLAR_ACCOUNT:-agriresolve-admin}"
RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"

echo "=========================================================="
echo "  AgriResolve Protocol - Testnet Smart Contract Deployment"
echo "=========================================================="
echo "Network            : $NETWORK"
echo "RPC URL            : $RPC_URL"
echo "Source Account     : $SOURCE_ACCOUNT"
echo "=========================================================="

if ! command -v stellar &> /dev/null; then
    echo "ERROR: 'stellar' CLI is required. Install via: cargo install --locked stellar-cli"
    exit 1
fi

echo "==> Building contracts for wasm32v1-none target..."
stellar contract build

REGISTRY_WASM="target/wasm32v1-none/release/agriresolve_registry.wasm"
ESCROW_WASM="target/wasm32v1-none/release/agriresolve_escrow.wasm"
DISPUTE_WASM="target/wasm32v1-none/release/agriresolve_dispute.wasm"

echo "==> Deploying Registry Contract..."
REGISTRY_ID=$(stellar contract deploy \
    --wasm "$REGISTRY_WASM" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    --alias agriresolve-registry)
echo "    Registry Contract ID: $REGISTRY_ID"

echo "==> Deploying Escrow Contract..."
ESCROW_ID=$(stellar contract deploy \
    --wasm "$ESCROW_WASM" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    --alias agriresolve-escrow)
echo "    Escrow Contract ID  : $ESCROW_ID"

echo "==> Deploying Dispute Contract..."
DISPUTE_ID=$(stellar contract deploy \
    --wasm "$DISPUTE_WASM" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    --alias agriresolve-dispute)
echo "    Dispute Contract ID : $DISPUTE_ID"

echo "==> Initializing Contracts..."
ADMIN_ADDR=$(stellar keys address "$SOURCE_ACCOUNT")

echo "    Initializing Registry with Admin: $ADMIN_ADDR"
stellar contract invoke \
    --id "$REGISTRY_ID" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    -- initialize --admin "$ADMIN_ADDR"

echo "    Initializing Escrow with Admin: $ADMIN_ADDR"
stellar contract invoke \
    --id "$ESCROW_ID" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    -- initialize --admin "$ADMIN_ADDR"

echo "    Initializing Dispute Contract with Admin: $ADMIN_ADDR and Escrow: $ESCROW_ID"
stellar contract invoke \
    --id "$DISPUTE_ID" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    -- initialize --admin "$ADMIN_ADDR" --escrow_contract "$ESCROW_ID"

echo "    Linking Dispute Contract to Escrow Engine..."
stellar contract invoke \
    --id "$ESCROW_ID" \
    --source-account "$SOURCE_ACCOUNT" \
    --network "$NETWORK" \
    -- set_dispute_contract --caller "$ADMIN_ADDR" --dispute_contract "$DISPUTE_ID"

ENV_OUT=".env.contracts"
cat <<EOF > "$ENV_OUT"
# AgriResolve Deployed Contract Identifiers
NEXT_PUBLIC_STELLAR_NETWORK=$NETWORK
NEXT_PUBLIC_STELLAR_RPC_URL=$RPC_URL
NEXT_PUBLIC_REGISTRY_CONTRACT_ID=$REGISTRY_ID
NEXT_PUBLIC_ESCROW_CONTRACT_ID=$ESCROW_ID
NEXT_PUBLIC_DISPUTE_CONTRACT_ID=$DISPUTE_ID
NEXT_PUBLIC_USDC_ADDRESS=CCQGOTGN6HTQE74XYERYSYVU57T5NPYWV5DLCROMB56OXTV6RSO6KIGK
EOF

echo "=========================================================="
echo "🎉 Deployment & Cross-Contract Linking Complete!"
echo "Saved environment variables to $ENV_OUT"
echo "=========================================================="
