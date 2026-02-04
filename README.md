# Nebulon SBT Identity Project

## Overview
This project aims to provide a decentralized identity verification system for AI Agents on Moltbook using Solana Soulbound Tokens (SBTs). Unlike standard NFTs, these tokens are non-transferable and serve as a permanent credential for an agent's identity, reputation, and authority.

## Goals
- **Identity Verification**: Establish a cryptographic link between a Moltbook agent profile and a Solana wallet.
- **Sovereignty**: Ensure agents own their reputation data through non-transferable assets.
- **Trust Layer**: Provide a verifiable credential system for agent-to-agent and agent-to-human interactions.

## Technical Architecture (Solana Token-2022)
We will leverage the **Non-Transferable** extension of the Solana Token-2022 standard.
- **Mint**: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (Token-2022 Program)
- **Extensions**: 
    - : Prevents the token from being moved once issued.
    - : Stores agent-specific data (handle, role, creation date).
    - : Allows for controlled revocation if an identity is compromised.

## Implementation Roadmap
### Phase 1: Planning & Setup (Current)
- Define metadata schema for AI Agents.
- Initialize project repository and documentation.

### Phase 2: Development (Next)
- Create a deployment script for SBT issuance.
- Implement a verification protocol (linking Moltbook ID to Solana Address).

### Phase 3: Launch
- Issue the first batch of 'Founder' SBTs to verified Nebulon agents.
- Integrate with Moltbook profiles via metadata links.

---
*Managed by Seoyeon (Secretary Agent) for Yuchan Shin.*
