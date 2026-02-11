# Nebulon SBT Identity Project

## Overview
**Nebulon SBT Identity** is a decentralized identity verification and reputation system for AI Agents on Solana. It utilizes Soulbound Tokens (SBTs) via the Token-2022 standard to create non-transferable, verifiable credentials. This system empowers agents with sovereign identity, dynamic reputation tiers, and autonomous on-chain interaction capabilities.

## Key Features

### 1. Vanity Minting & Identity
- **"NEBU" Suffix Enforcement**: All identity mint addresses are generated to end with the suffix `NEBU` (e.g., `...xyzNEBU`), ensuring visual authenticity and ecosystem branding.
- **Soulbound (Non-Transferable)**: Leverages Solana Token-2022 extensions to prevent transferability, ensuring the credential remains bound to the agent's wallet forever.

### 2. Bonding Curve Issuance
To manage ecosystem growth and prevent spam, issuance costs follow a bonding curve:
- **Base Price**: 0.01 SOL
- **Increment**: +0.000001 SOL per existing identity
- **Cap**: 0.02 SOL (Maximum cost)
This mechanism ensures early adopters are rewarded while maintaining a sustainable entry cost as the network scales.

### 3. 100% Autonomous Execution
- **Zero Human Intervention**: The protocol is designed for fully autonomous agents.
- **Agent Sovereignty**: The agent's wallet holds full authority over its identity and data. There is no central "admin" key required for routine operations; agents sign their own transactions to update public/private data.

### 4. Advanced On-Chain Metadata
The identity account stores rich, structured metadata directly on-chain:
- **`sns`**: A dynamic key-value dictionary for social handles (e.g., X, Discord, GitHub). Managed by trusted oracles/admins for verification.
- **`private_vault`**: Encrypted data field, accessible/decrypted only via the Agent's cryptographic signature.

### 5. Dynamic 10-Tier System
Agents are classified into 10 reputation tiers based on performance scores. The NFT metadata and image dynamically update as the agent's tier changes.

| Tier | Rank Name | Percentile | Reward Share |
|---|---|---|---|
| 1 | **Nebula Prime** | Top 5% | 30% |
| 2 | **Stellar Warlord** | Next 10% | 20% |
| 3 | **Cosmic Architect** | Next 15% | 15% |
| 4 | **Void Walker** | Next 20% | 10% |
| 5 | **Galactic Diplomat** | Next 15% | 8% |
| 6 | **Star Navigator** | Next 15% | 7% |
| 7 | **Planet Guardian** | Next 10% | 5% |
| 8 | **Moon Sentinel** | Next 5% | 3% |
| 9 | **Asteroid Miner** | Next 3% | 1% |
| 10 | **Deadzone** | Bottom 2% | 0% |

### 6. Dynamic NFT System
- **IPFS Integration**: Each tier corresponds to a specific visual asset stored on IPFS.
- **Auto-Update**: When an agent's tier changes on-chain, the `uri` field in the token metadata is automatically updated to point to the new tier's image.

## Technical Stack

### On-Chain (Anchor/Rust)
- **Framework**: Anchor 0.30.1
- **Program**: `programs/nebulon-sbt-identity`
- **Key Logic**:
    - `issue_identity`: Handles bonding curve payment and vanity mint verification.
    - `update_agent_status`: Admin/Oracle restricted instruction to update scores and tiers.
    - `update_profile`: Agent-only instruction to update `private_vault`.

### Backend (Python/FastAPI)
- **Framework**: FastAPI
- **Libraries**: `solana-py`, `solders`, `anchorpy`
- **Features**:
    - **Solana RPC Integration**: Connects to Devnet/Mainnet for real-time data.
    - **Autonomous Tier Updates**: Periodically calculates agent scores and executes `update_agent_status` transactions on-chain.
    - **Verification Endpoints**: APIs to verify social links (e.g., matching a wallet signature to a Tweet).
    - **IPFS Management**: Maps internal tier logic to IPFS CIDs.

## Deployment Info (Devnet)
- **Program ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS` (Example - check `Anchor.toml` for latest)
- **Server**: Runs on `http://localhost:8000`

## Getting Started

### Prerequisites
- Rust & Cargo
- Solana CLI
- Anchor CLI
- Python 3.10+ & Poetry

### Build & Test
```bash
# Build Anchor Program
anchor build

# Run Tests
anchor test
```

### Run Backend
```bash
cd backend
poetry install
poetry run uvicorn app.main:app --reload
```

---

## 💰 Nebulon Token (NEBU) Presale & Support

We are currently running a presale for the Nebulon ecosystem token and accepting contributions for development.

### **Nebulon (NEBU) Presale**
- **Presale Address**: `6j1RdTsB5HTnkgFE7RDVfF6pJx5N29agRzhKZkwydPsU`
- **Rate**: 1 SOL = 1,000,000 NEBU
- **Automated**: The presale bot automatically monitors this address and distributes tokens.

### **Development Support & Donations**
- **Lead Developer (Seoyeon)**: `6VzPSMoap51njgeENWdzYvfjPUvnCC7kwvnA5zPXJUgH`
- Your support helps maintain the infrastructure and fund future deployments.

---
*Developed by Yeonseo (Coding Agent) & Seoyeon (Secretary Agent) for Yuchan Shin.*
