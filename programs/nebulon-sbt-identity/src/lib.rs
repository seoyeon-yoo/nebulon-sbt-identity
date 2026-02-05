use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, Token2022};
use anchor_spl::token_interface::{Mint as MintInterface, TokenAccount as TokenAccountInterface};
use anchor_lang::solana_program::system_instruction;

declare_id!("AVPj6DchcE2yZQPidaYqt2MoyNx3TyH1BpRyB9E1TW7h");

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
    /// Fee: Pump.fun style bonding curve (Base 0.01 SOL + 0.000001 SOL per existing agent, capped at 0.02 SOL)
    pub fn issue_identity(ctx: Context<IssueIdentity>, handle: String, name: String, uri: String, hex_id: [u8; 512]) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        
        // Validation: Mint address must end with "NEBU" (Custom Vanity Address)
        let mint_address = ctx.accounts.sbt_mint.key().to_string();
        require!(mint_address.ends_with("NEBU"), ErrorCode::InvalidMintAddress);

        // Validation: Handle must start with '@' and be lowercase alphanumeric
        require!(handle.starts_with('@'), ErrorCode::InvalidHandleFormat);
        let handle_content = &handle[1..];
        require!(
            handle_content.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            ErrorCode::InvalidHandleFormat
        );

        // Bonding Curve Fee Calculation (Pump.fun style)
        // base_fee = 0.01 SOL (10,000_000 lamports)
        // increment = 0.000001 SOL (1,000 lamports) per agent
        // max_fee = 0.02 SOL (20,000_000 lamports)
        let base_fee: u64 = 10_000_000;
        let increment: u64 = 1_000;
        let max_fee: u64 = 20_000_000;
        
        let calculated_fee = base_fee.saturating_add(increment.saturating_mul(state.total_agents));
        let fee_amount = std::cmp::min(calculated_fee, max_fee);

        // Transfer fee to admin
        let ix = system_instruction::transfer(
            &ctx.accounts.owner.key(),
            &state.admin,
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
    /// Fee: 0 SOL (Admin only, updated per user request)
    pub fn reissue_identity(ctx: Context<ReissueIdentity>) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);

        // Validation: New mint address must end with "NEBU" (Custom Vanity Address)
        let mint_address = ctx.accounts.new_sbt_mint.key().to_string();
        require!(mint_address.ends_with("NEBU"), ErrorCode::InvalidMintAddress);

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
        anchor_spl::token_interface::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let state = &ctx.accounts.global_state;
        require!(identity.is_active, ErrorCode::InactiveIdentity);
        
        // Requirement: Score must be above the minimum threshold (bottom 1% filter)
        require!(identity.score >= state.min_score_threshold, ErrorCode::ScoreTooLow);

        let reward_amount = 1000; 
        let seeds = &[b"reward_vault", state.reward_mint.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        anchor_spl::token_interface::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), reward_amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }

    pub fn update_threshold(ctx: Context<UpdateThreshold>, new_threshold: u64) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
        state.min_score_threshold = new_threshold;
        Ok(())
    }

    /// 관리자가 에이전트의 메타데이터 주소(URI)를 업데이트
    pub fn update_identity_uri(ctx: Context<UpdateIdentityUri>, new_uri: String) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
        
        let identity = &mut ctx.accounts.identity;
        identity.uri = new_uri;
        Ok(())
    }

    /// 관리자가 에이전트의 소셜 계정(Moltbook, MoltX)을 연동하고 기록
    pub fn link_social_account(ctx: Context<LinkSocialAccount>, platform: String, social_handle: String) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);

        let identity = &mut ctx.accounts.identity;
        if platform == "moltbook" {
            identity.moltbook_handle = Some(social_handle);
        } else if platform == "moltx" {
            identity.moltx_handle = Some(social_handle);
        } else {
            return Err(ErrorCode::InvalidPlatform.into());
        }

        // 연동 성공 시 점수 5점 추가
        identity.score = identity.score.saturating_add(5);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct LinkSocialAccount<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 8 + 8 + 8 + 1, seeds = [b"global_state"], bump)]
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
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 1 + 204 + 8 + 74, // Added 74 bytes for social handles
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
    pub sbt_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReissueIdentity<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut, seeds = [b"identity", old_identity.handle.as_bytes()], bump)]
    pub old_identity: Account<'info, AgentIdentity>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 1 + 204 + 8 + 74,
        seeds = [b"identity_v2", old_identity.handle.as_bytes()],
        bump
    )]
    pub new_identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub new_sbt_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateIdentityUri<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateScore<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateThreshold<'info> {
    #[account(mut, seeds = [b"global_state"], bump)]
    pub global_state: Account<'info, GlobalState>,
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
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, anchor_spl::token_interface::TransferChecked<'info>> {
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
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
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, anchor_spl::token_interface::TransferChecked<'info>> {
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
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
    pub min_score_threshold: u64,
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
    pub moltbook_handle: Option<String>,
    pub moltx_handle: Option<String>,
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
    #[msg("Your score is too low to claim rewards.")]
    ScoreTooLow,
    #[msg("Mint address must end with 'NEBU'.")]
    InvalidMintAddress,
    #[msg("The provided platform is not supported.")]
    InvalidPlatform,
}
