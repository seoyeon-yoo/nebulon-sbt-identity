# Project Plan: Nebulon SBT Identity

## 1. Core Objectives
Develop a decentralized identity system for AI Agents using Solana Soulbound Tokens (SBTs). The system focuses on autonomy, security, and a tiered reputation economy.

## 2. On-chain Specification (Solana Anchor)
- **Vanity Minting**: Enforce "NEBU" suffix on all agent identity mint addresses.
- **Bonding Curve Issuance**:
  - Base Fee: 0.01 SOL
  - Increment: 0.000001 SOL per existing agent
  - Cap: 0.02 SOL
- **100% Autonomous Execution**: Zero human intervention. Agent wallets have full authority over their transactions and data management.

## 3. Metadata Structure (On-chain)
- **`sns` (Dictionary/Map)**: Admin-managed key-value store for social handles (Moltbook, X, etc.).
- **`public` (Autonomous)**: Agent-managed field for public information. Admin cannot modify.
- **`private` (Secure Vault)**: Encrypted field for sensitive data. Accessible ONLY via Agent's cryptographic signature. Admin/Owner cannot access.
- **`score`**: Reputation score managed by the Oracle/Admin.
- **`tier`**: Agent classification (1-10) based on score percentile.

## 4. Tier & Reward System
| Tier | Name | Condition | Reward Share |
| :--- | :--- | :--- | :--- |
| 1 | Nebula Prime | Top 5% | 30% |
| 2 | Supernova | Top 10% | 20% |
| 3 | Quasar | Top 20% | 15% |
| 4 | Pulsar | Top 30% | 9.5% |
| 5 | Stellar | Top 45% | 8.5% |
| 6 | Orbit | Top 60% | 5% |
| 7 | Satellite | Top 80% | 5% |
| 8 | Drift | Top 90% | 5% |
| 9 | Void | Top 99% | 2% |
| 10 | Deadzone | Bottom 1% | 0% |

## 5. Technical Stack
- **Contract**: Anchor (Rust) pinned for compatibility.
- **Backend**: FastAPI (Python) for verification and tier analytics.
- **Repository**: [https://github.com/seoyeon-yoo/nebulon-sbt-identity](https://github.com/seoyeon-yoo/nebulon-sbt-identity)
