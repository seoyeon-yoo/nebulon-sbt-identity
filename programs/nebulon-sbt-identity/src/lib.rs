use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token_interface::{Mint, TokenAccount, Token2022};
use anchor_spl::token_2022::spl_token_2022::extension::{
    ExtensionType,
    non_transferable::NonTransferable,
};

declare_id!("Neb1SBT111111111111111111111111111111111111");

#[program]
pub mod nebulon_sbt_identity {
    use super::*;

    /// Initialize the Global Registry State
    pub fn initialize_registry(ctx: Context<InitializeRegistry>, scoring_authority: Pubkey) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.admin = ctx.accounts.admin.key();
        registry.scoring_authority = scoring_authority;
        registry.total_agents = 0;
        registry.yield_pool_balance = 0;
        Ok(())
    }

    /// Issue a new Soulbound Token (SBT)
    /// Cost: 0.01 SOL (transferred to treasury)
    pub fn issue_sbt(ctx: Context<IssueSbt>, handle: String) -> Result<()> {
        let clock = Clock::get()?;
        let identity = &mut ctx.accounts.identity;
        
        // 1. Charge Issuance Fee (0.01 SOL)
        let fee = 10_000_000; // 0.01 SOL in lamports
        let ix = system_instruction::transfer(
            &ctx.accounts.agent.key(),
            &ctx.accounts.treasury.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[ctx.accounts.agent.to_account_info(), ctx.accounts.treasury.to_account_info()],
        )?;

        // 2. Initialize Identity Account
        identity.agent = ctx.accounts.agent.key();
        identity.handle = handle;
        identity.score = 0;
        identity.active_mint = ctx.accounts.mint.key();
        identity.created_at = clock.unix_timestamp;
        identity.is_active = true;

        let registry = &mut ctx.accounts.registry;
        registry.total_agents += 1;

        msg!("SBT Issued for Agent: {}", identity.handle);
        Ok(())
    }

    /// Reissue an SBT (Update Identity)
    /// Cost: 0.005 SOL
    pub fn reissue_sbt(ctx: Context<ReissueSbt>) -> Result<()> {
        // 1. Charge Reissuance Fee (0.005 SOL)
        let fee = 5_000_000; // 0.005 SOL
        let ix = system_instruction::transfer(
            &ctx.accounts.agent.key(),
            &ctx.accounts.treasury.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[ctx.accounts.agent.to_account_info(), ctx.accounts.treasury.to_account_info()],
        )?;

        // 2. Update Identity: Mark old mint as inactive
        let identity = &mut ctx.accounts.identity;
        identity.active_mint = ctx.accounts.new_mint.key();
        
        msg!("SBT Reissued for Agent: {}", identity.handle);
        Ok(())
    }

    /// Update Agent Score (Only Scoring Authority)
    pub fn update_score(ctx: Context<UpdateScore>, new_score: u64) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        identity.score = new_score;
        Ok(())
    }

    /// Mock Distribution logic (to be integrated with NEBU token)
    pub fn distribute_yield(ctx: Context<DistributeYield>) -> Result<()> {
        // Logic for Tiered distribution:
        // 1. Identify Top 10% agents (would require sorting or off-chain score aggregation in practice)
        // 2. Allocate 50% of yield pool to Top 10%
        // 3. Allocate 50% to the rest
        msg!("Distributing NEBU rewards based on tiered scores.");
        Ok(())
    }
}

#[account]
pub struct RegistryState {
    pub admin: Pubkey,
    pub scoring_authority: Pubkey,
    pub total_agents: u64,
    pub yield_pool_balance: u64,
}

#[account]
pub struct AgentIdentity {
    pub agent: Pubkey,
    pub handle: String,
    pub score: u64,
    pub active_mint: Pubkey,
    pub created_at: i64,
    pub is_active: bool,
}

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 8 + 8)]
    pub registry: Account<'info, RegistryState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(handle: String)]
pub struct IssueSbt<'info> {
    #[account(
        init, 
        payer = agent, 
        space = 8 + 32 + 64 + 8 + 32 + 8 + 1,
        seeds = [b"identity", agent.key().as_ref()],
        bump
    )]
    pub identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub agent: Signer<'info>,
    /// CHECK: Treasury account to receive fees
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    #[account(mut)]
    pub registry: Account<'info, RegistryState>,
    /// The Soulbound Mint (Token-2022)
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReissueSbt<'info> {
    #[account(mut, seeds = [b"identity", agent.key().as_ref()], bump)]
    pub identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub agent: Signer<'info>,
    /// CHECK: Treasury
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    pub new_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateScore<'info> {
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub authority: Signer<'info>,
    #[account(has_one = scoring_authority @ ErrorCode::Unauthorized)]
    pub registry: Account<'info, RegistryState>,
}

#[derive(Accounts)]
pub struct DistributeYield<'info> {
    #[account(mut)]
    pub registry: Account<'info, RegistryState>,
    pub admin: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
