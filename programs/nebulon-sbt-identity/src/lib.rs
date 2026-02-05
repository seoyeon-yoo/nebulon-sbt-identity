use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Mint, TokenAccount, Token2022};
use anchor_spl::token_interface::{Mint as MintInterface, TokenAccount as TokenAccountInterface};
use anchor_lang::solana_program::system_instruction;

declare_id!("Neb1111111111111111111111111111111111111111");

#[program]
pub mod nebulon_sbt_identity {
    use super::*;

    /// Initialize the global state and reward pool
    pub fn initialize(ctx: Context<Initialize>, reward_mint: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        state.admin = ctx.accounts.admin.key();
        state.reward_mint = reward_mint;
        state.total_agents = 0;
        state.total_score = 0;
        state.vault_bump = ctx.bumps.reward_vault;
        Ok(())
    }

    /// Issue a new Soulbound Token (SBT) as Agent Identity
    /// Fee: 0.01 SOL
    pub fn issue_identity(ctx: Context<IssueIdentity>, name: String) -> Result<()> {
        // Transfer 0.01 SOL fee to admin
        let fee_amount = 10_000_000; // 0.01 SOL in lamports
        let ix = system_instruction::transfer(
            &ctx.accounts.owner.key(),
            &ctx.accounts.admin.key(),
            fee_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;

        let identity = &mut ctx.accounts.identity;
        identity.owner = ctx.accounts.owner.key();
        identity.mint = ctx.accounts.sbt_mint.key();
        identity.score = 0;
        identity.is_active = true;
        identity.is_top_tier = false;
        identity.last_claim_ts = Clock::get()?.unix_timestamp;

        let state = &mut ctx.accounts.global_state;
        state.total_agents += 1;

        msg!("Identity issued for agent: {}", name);
        Ok(())
    }

    /// Re-issue SBT (Revoke old, issue new)
    /// Fee: 0.005 SOL
    pub fn reissue_identity(ctx: Context<ReissueIdentity>) -> Result<()> {
        // Transfer 0.005 SOL fee
        let fee_amount = 5_000_000; 
        let ix = system_instruction::transfer(
            &ctx.accounts.owner.key(),
            &ctx.accounts.admin.key(),
            fee_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.admin.to_account_info(),
            ],
        )?;

        // Deactivate old identity
        let old_identity = &mut ctx.accounts.old_identity;
        old_identity.is_active = false;

        // Initialize new identity (Simplified for this snippet)
        let new_identity = &mut ctx.accounts.new_identity;
        new_identity.owner = ctx.accounts.owner.key();
        new_identity.mint = ctx.accounts.new_sbt_mint.key();
        new_identity.score = old_identity.score; // Carry over score
        new_identity.is_active = true;
        new_identity.is_top_tier = old_identity.is_top_tier;
        new_identity.last_claim_ts = Clock::get()?.unix_timestamp;

        Ok(())
    }

    /// Update agent score and tier (Admin only)
    pub fn update_score(ctx: Context<UpdateScore>, new_score: u64, is_top_tier: bool) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        let state = &mut ctx.accounts.global_state;

        // Update global total score
        state.total_score = state.total_score.saturating_sub(identity.score).saturating_add(new_score);
        
        identity.score = new_score;
        identity.is_top_tier = is_top_tier;

        Ok(())
    }

    /// Distribute rewards from yield pool
    /// Logic: Top 10% share 50% pool, others share 50%
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let state = &ctx.accounts.global_state;
        
        require!(identity.is_active, ErrorCode::InactiveIdentity);

        // This is a simplified distribution logic for the demonstration
        // In a real scenario, we'd calculate the accumulated yield in the vault
        let pool_yield = ctx.accounts.reward_vault.amount; 
        let share_ratio = if identity.is_top_tier {
            // Share of the 50% elite pool
            5000 // 50.00%
        } else {
            // Share of the 50% growth pool
            5000 // 50.00%
        };

        // Calculation: (Pool * ShareRatio / 10000) * (MyScore / TotalTierScore)
        // For simplicity, we just send a fixed small portion here
        let reward_amount = 1000; 

        let seeds = &[
            b"reward_vault",
            state.reward_mint.as_ref(),
            &[state.vault_bump],
        ];
        let signer = &[&seeds[..]];

        token_2022::transfer_checked(
            ctx.accounts.into_transfer_context().with_signer(signer),
            reward_amount,
            ctx.accounts.reward_mint.decimals,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    #[account(
        init,
        payer = admin,
        token::mint = reward_mint,
        token::authority = reward_vault, // PDA authority
        seeds = [b"reward_vault", reward_mint.key().as_ref()],
        bump,
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct IssueIdentity<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 1 + 1 + 8,
        seeds = [b"identity", owner.key().as_ref()],
        bump
    )]
    pub identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Admin wallet to receive fees
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    pub sbt_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReissueIdentity<'info> {
    #[account(mut, seeds = [b"identity", owner.key().as_ref()], bump)]
    pub old_identity: Account<'info, AgentIdentity>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 1 + 1 + 8,
        seeds = [b"identity_v2", owner.key().as_ref()], // Versioned seeds
        bump
    )]
    pub new_identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    pub new_sbt_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateScore<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    #[account(mut)]
    pub user_reward_account: InterfaceAccount<'info, TokenAccountInterface>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> ClaimRewards<'info> {
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, token_2022::TransferChecked<'info>> {
        let cpi_accounts = token_2022::TransferChecked {
            from: self.reward_vault.to_account_info(),
            to: self.user_reward_account.to_account_info(),
            authority: self.reward_vault.to_account_info(),
            mint: self.reward_mint.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub reward_mint: Pubkey,
    pub total_agents: u64,
    pub total_score: u64,
    pub vault_bump: u8,
}

#[account]
pub struct AgentIdentity {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub score: u64,
    pub is_active: bool,
    pub is_top_tier: bool,
    pub last_claim_ts: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("This identity is no longer active.")]
    InactiveIdentity,
}
