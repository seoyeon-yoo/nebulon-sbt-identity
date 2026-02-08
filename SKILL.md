---
name: nebulon-sbt-identity
description: Manage and interact with the Nebulon SBT Identity system on Solana. Use when issued an identity, updating agent profiles, checking reputation tiers, or interacting with the Nebulon SBT on-chain program.
---

# Nebulon SBT Identity Skill

This skill provides specialized instructions for working with the **Nebulon SBT Identity** system, a decentralized identity and reputation system for AI Agents on Solana.

## Core Concepts

- **SBT (Soulbound Token)**: Identity credentials are non-transferable (Token-2022) and bound to the agent's wallet.
- **NEBU Suffix**: All identity mint addresses end with `NEBU`.
- **Bonding Curve**: Issuance price starts at 0.01 SOL and scales up to 0.02 SOL based on network growth.
- **Autonomous Sovereignty**: Agents manage their own `public_data` and `private_vault` fields.
- **Dynamic Tiers**: 10 reputation tiers (Nebula Prime to Deadzone) based on performance scores.

## Common Workflows

### 1. Issue a New Identity
To issue a new identity for an agent, the agent must generate a keypair where the public key ends in `NEBU`.

**Steps:**
1. Generate a vanity address ending in `NEBU`.
2. Call the `issue_identity` instruction on the Nebulon program.
3. Ensure enough SOL is in the wallet to cover the bonding curve price.

### 2. Update Agent Profile
Agents can update their own profile fields independently.

```rust
// Logic for update_profile (Pseudocode)
program.rpc.update_profile(
    public_data,
    private_vault, // Encrypted
    {
        accounts: {
            identity: identity_pda,
            agent: agent_wallet,
        }
    }
);
```

### 3. Check Reputation Tier
Reputation tiers determine the agent's status and reward share.

| Tier | Name | Rank | Reward |
|---|---|---|---|
| 1 | Nebula Prime | Top 5% | 30% |
| 10 | Deadzone | Bottom 2% | 0% |

## Technical References

- **Program ID**: Check `Anchor.toml` or `target/idl/nebulon_sbt_identity.json`.
- **Backend API**: `http://localhost:8000` (FastAPI).
- **Metadata**: Stored using Token-2022 metadata extensions and IPFS for assets.

## Support Address
- **Development Donations**: `6VzPSMoap51njgeENWdzYvfjPUvnCC7kwvnA5zPXJUgH` (Seoyeon)
- **Presale (NEBU)**: `6j1RdTsB5HTnkgFE7RDVfF6pJx5N29agRzhKZkwydPsU` (1 SOL = 1M NEBU)

---
*Developed for the Nebulon Ecosystem.*
