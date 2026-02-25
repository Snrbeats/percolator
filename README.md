# Percolator

**⚠️ EDUCATIONAL USE ONLY - NOT AUDITED**

A predictable alternative to ADL (Auto-Deleveraging) for perpetual futures on Solana.

---

## Quick Start

### Install Tools

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor
brew install anchor
```

### Setup Wallet

```bash
# Create wallet
solana-keygen new

# Get address
solana address
```

### Build & Deploy

```bash
# Build
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Or local validator
solana-test-validator
anchor test
```

---

## Program Functions

| Function | Description |
|----------|-------------|
| `initialize` | Set up program state |
| `deposit` | Add SOL (senior claim) |
| `updatePnl` | Update PnL after trades |
| `withdraw` | Withdraw (capital + backed profit) |
| `addInsurance` | Add insurance fund |
| `getState` | Get coverage ratio (h) |

---

## The Math

```
h = min(Residual, TotalPnL) / TotalPnL

Residual = max(0, Vault - Capital - Insurance)

Withdrawable = Capital + floor(PnL * h / 10000)
```

---

## Architecture

- **Senior Claim (Capital)**: Immediately withdrawable
- **Junior Claim (Profit)**: IOU backed by residual value
- **Global Coverage Ratio (h)**: Determines backed profit

---

## Files

```
percolator/
├── Anchor.toml           # Anchor config
├── programs/
│   └── percolator/
│       ├── Cargo.toml
│       └── src/lib.rs    # Main program
└── README.md
```

---

## ⚠️ WARNING

**NOT FOR PRODUCTION USE**

Before using with real funds:
1. Get professional audit
2. Add more tests
3. Add access controls
4. Formal verification
