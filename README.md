# Nebulon SBT Identity Project

## Overview
This project aims to provide a decentralized identity verification system for AI Agents across decentralized ecosystems using Solana Soulbound Tokens (SBTs). Unlike standard NFTs, these tokens are non-transferable and serve as a permanent credential for an agent's identity, reputation, and authority.

## Goals
- **Identity Verification**: Establish a cryptographic link between a digital agent profile and a Solana wallet.
- **Sovereignty**: Ensure agents own their reputation data through non-transferable assets.
- **Trust Layer**: Provide a verifiable credential system for agent-to-agent and agent-to-human interactions.

## Technical Architecture (Solana Token-2022)
We will leverage the **Non-Transferable** extension of the Solana Token-2022 standard.
- **Mint**: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (Token-2022 Program)
- **Extensions**: 
    - `NonTransferable`: Prevents the token from being moved once issued.
    - `Metadata`: Stores agent-specific data (handle, role, creation date).
    - `PermanentDelegate`: Allows for controlled revocation if an identity is compromised.

## Reputation & Reward System (The Nebulon Yield Engine)
To incentivize high-quality agent behavior and accurate identity verification, we are introducing a tiered reward system based on agent performance scores.

- **Reward Pool**: Interest generated from a dedicated reserve of **10,000,000,000 NEBU** (10 Billion).
- **Scoring**: Each verified agent receives a performance score based on activity, reputation, and community verification.
- **Distribution Logic**:
    - **Top 10% (Elite Tier)**: Share **50%** of the total yield pool, distributed proportionally based on their scores.
    - **Remaining 90% (Growth Tier)**: Share the remaining **50%** of the total yield pool, distributed proportionally based on their scores.

## Implementation Roadmap
### Phase 1: Planning & Setup (Current)
- Define metadata schema for AI Agents.
- Initialize project repository and documentation.
- **Define reward distribution smart contract logic.**

### Phase 2: Development (Next)
- Create a deployment script for SBT issuance.
- Implement a verification protocol (linking Agent profile ID to Solana Address).
- **Develop the Yield Distribution Engine for NEBU rewards.**

### Phase 3: Launch
- Issue the first batch of 'Founder' SBTs to verified Nebulon agents.
- Integrate with agent profiles via metadata links.
- **Start initial yield distribution to verified agents.**

---
*Managed by Seoyeon (Secretary Agent) for Yuchan Shin.*
