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
    /// Requirement: Handle (@handle), NFT Metadata URL (URI), 512-byte Hex ID
    /// Fee: 0.01 SOL
    pub fn issue_identity(ctx: Context<IssueIdentity>, handle: String, name: String, uri: String, hex_id: [u8; 512]) -> Result<()> {
        // Validation: Handle must start with '@' and be lowercase alphanumeric
        require!(handle.starts_with('@'), ErrorCode::InvalidHandleFormat);
        let handle_content = &handle[1..];
        require!(
            handle_content.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            ErrorCode::InvalidHandleFormat
        );

        // Transfer 0.01 SOL fee to admin
        let fee_amount = 10_000_000; 
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
        identity.handle = handle;
        identity.hex_id = hex_id;
        identity.score = 0;
        identity.is_active = true;
        identity.is_top_tier = false;
        identity.uri = uri;
        identity.last_claim_ts = Clock::get()?.unix_timestamp;

        let state = &mut ctx.accounts.global_state;
        state.total_agents += 1;

        msg!("Identity issued for agent: {} with handle {}", name, identity.handle);
        Ok(())
    }

    /// Re-issue SBT (Revoke old, issue new)
    /// Handle and Hex ID cannot be changed.
    /// Fee: 0.005 SOL
    pub fn reissue_identity(ctx: Context<ReissueIdentity>) -> Result<()> {
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

        let old_identity = &mut ctx.accounts.old_identity;
        old_identity.is_active = false;

        let new_identity = &mut ctx.accounts.new_identity;
        new_identity.owner = ctx.accounts.owner.key();
        new_identity.mint = ctx.accounts.new_sbt_mint.key();
        new_identity.handle = old_identity.handle.clone(); // Handle is immutable
        new_identity.hex_id = old_identity.hex_id;         // Hex ID is immutable
        new_identity.score = old_identity.score;
        new_identity.is_active = true;
        new_identity.is_top_tier = old_identity.is_top_tier;
        new_identity.uri = old_identity.uri.clone();
        new_identity.last_claim_ts = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn update_score(ctx: Context<UpdateScore>, new_score: u64, is_top_tier: bool) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        let state = &mut ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);

        state.total_score = state.total_score.saturating_sub(identity.score).saturating_add(new_score);
        identity.score = new_score;
        identity.is_top_tier = is_top_tier;
        Ok(())
    }

    pub fn withdraw_sol(ctx: Context<WithdrawAdmin>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
        **ctx.accounts.global_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }

    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
        let seeds = &[b"reward_vault", state.reward_mint.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        token_2022::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let state = &ctx.accounts.global_state;
        require!(identity.is_active, ErrorCode::InactiveIdentity);
        let reward_amount = 1000; 
        let seeds = &[b"reward_vault", state.reward_mint.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        token_2022::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), reward_amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 8 + 8 + 1, seeds = [b"global_state"], bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    #[account(init, payer = admin, token::mint = reward_mint, token::authority = reward_vault, seeds = [b"reward_vault", reward_mint.key().as_ref()], bump)]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
#[instruction(handle: String)]
pub struct IssueIdentity<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 1 + 204 + 8, // Space for 512-byte Hex ID
        seeds = [b"identity", handle.as_bytes()], 
        bump
    )]
    pub identity: Account<'info, AgentIdentity>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32,
        seeds = [b"owner_mapping", owner.key().as_ref()], 
        bump
    )]
    pub owner_map: Account<'info, OwnerMapping>,
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
    #[account(mut, seeds = [b"identity", old_identity.handle.as_bytes()], bump)]
    pub old_identity: Account<'info, AgentIdentity>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 1 + 204 + 8,
        seeds = [b"identity_v2", old_identity.handle.as_bytes()],
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
pub struct WithdrawAdmin<'info> {
    #[account(mut, seeds = [b"global_state"], bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    #[account(mut)]
    pub admin_token_account: InterfaceAccount<'info, TokenAccountInterface>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> WithdrawTokens<'info> {
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, token_2022::TransferChecked<'info>> {
        let cpi_accounts = token_2022::TransferChecked {
            from: self.reward_vault.to_account_info(),
            to: self.admin_token_account.to_account_info(),
            authority: self.reward_vault.to_account_info(),
            mint: self.reward_mint.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
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
    pub handle: String,    
    pub hex_id: [u8; 512], // 512-byte Hex ID
    pub score: u64,
    pub is_active: bool,
    pub is_top_tier: bool,
    pub uri: String,
    pub last_claim_ts: i64,
}

#[account]
pub struct OwnerMapping {
    pub identity: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("This identity is no longer active.")]
    InactiveIdentity,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Handle must start with @ and contain only lowercase letters and digits.")]
    InvalidHandleFormat,
}
