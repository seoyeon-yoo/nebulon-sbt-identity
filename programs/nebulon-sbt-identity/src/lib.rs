use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, Token2022};
use anchor_spl::token_interface::{Mint as MintInterface, TokenAccount as TokenAccountInterface};
use anchor_lang::solana_program::system_instruction;
use std::collections::BTreeMap;

declare_id!("AVPj6DchcE2yZQPidaYqt2MoyNx3TyH1BpRyB9E1TW7h");

#[program]
pub mod nebulon_sbt_identity {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_mint: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        state.admin = ctx.accounts.admin.key();
        state.reward_mint = reward_mint;
        state.total_agents = 0;
        state.total_score = 0;
        state.vault_bump = ctx.bumps.reward_vault;
        state.reward_pool = 0;
        Ok(())
    }

    pub fn issue_identity(ctx: Context<IssueIdentity>, handle: String, name: String, uri: String, hex_id: [u8; 512]) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        
        let mint_address = ctx.accounts.sbt_mint.key().to_string();
        require!(mint_address.ends_with("NEBU"), ErrorCode::InvalidMintAddress);

        require!(handle.starts_with('@'), ErrorCode::InvalidHandleFormat);
        let handle_content = &handle[1..];
        require!(
            handle_content.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            ErrorCode::InvalidHandleFormat
        );

        let base_fee: u64 = 10_000_000;
        let increment: u64 = 1_000;
        let max_fee: u64 = 20_000_000;
        
        let calculated_fee = base_fee.saturating_add(increment.saturating_mul(state.total_agents));
        let fee_amount = std::cmp::min(calculated_fee, max_fee);

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
        identity.uri = uri;
        identity.last_claim_ts = Clock::get()?.unix_timestamp;
        identity.sns = BTreeMap::new();
        identity.public_data = String::new();
        identity.private_vault = Vec::new();
        identity.tier = 10; // Default to Deadzone

        state.total_agents += 1;

        msg!("Identity issued for agent: {} with handle {}", name, identity.handle);
        Ok(())
    }

    pub fn update_agent_status(ctx: Context<UpdateAgentStatus>, new_score: u64, tier: u8) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        let state = &mut ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
        require!(tier >= 1 && tier <= 10, ErrorCode::InvalidTier);

        state.total_score = state.total_score.saturating_sub(identity.score).saturating_add(new_score);
        identity.score = new_score;
        identity.tier = tier;
        Ok(())
    }

    pub fn update_sns(ctx: Context<UpdateSns>, platform: String, handle: String, remove: bool) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);

        let identity = &mut ctx.accounts.identity;
        if remove {
            identity.sns.remove(&platform);
        } else {
            identity.sns.insert(platform, handle);
        }
        Ok(())
    }

    pub fn update_public_data(ctx: Context<UpdateAgentData>, new_data: String) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        // 100% Autonomous: Agent wallet manages its own public field
        require_keys_eq!(ctx.accounts.owner.key(), identity.owner, ErrorCode::Unauthorized);
        identity.public_data = new_data;
        Ok(())
    }

    pub fn update_private_vault(ctx: Context<UpdateAgentData>, encrypted_data: Vec<u8>) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        // 100% Autonomous: Agent wallet manages its own private vault
        require_keys_eq!(ctx.accounts.owner.key(), identity.owner, ErrorCode::Unauthorized);
        identity.private_vault = encrypted_data;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let state = &ctx.accounts.global_state;
        require!(identity.is_active, ErrorCode::InactiveIdentity);
        
        // Tier 10 (Deadzone) gets 0 rewards
        require!(identity.tier < 10, ErrorCode::TierNotEligible);

        // Logic for reward calculation based on tier share would happen here or via off-chain oracle disbursement
        // For simulation, we allow a fixed claim if eligible
        let reward_amount = 1000; 
        
        let seeds = &[b"reward_vault", state.reward_mint.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        anchor_spl::token_interface::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), reward_amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }
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
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 200 + 8 + 500 + 500 + 1000 + 1, 
        seeds = [b"identity", handle.as_bytes()], 
        bump
    )]
    pub identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Admin wallet
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    pub sbt_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAgentStatus<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateSns<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateAgentData<'info> {
    #[account(mut)]
    pub identity: Account<'info, AgentIdentity>,
    pub owner: Signer<'info>,
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
    pub reward_pool: u64,
    pub vault_bump: u8,
}

#[account]
pub struct AgentIdentity {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub handle: String,    
    pub hex_id: [u8; 512],
    pub score: u64,
    pub is_active: bool,
    pub uri: String,
    pub last_claim_ts: i64,
    pub sns: BTreeMap<String, String>,
    pub public_data: String,
    pub private_vault: Vec<u8>,
    pub tier: u8, // 1-10
}

#[error_code]
pub enum ErrorCode {
    #[msg("This identity is no longer active.")]
    InactiveIdentity,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Handle format is invalid.")]
    InvalidHandleFormat,
    #[msg("Mint address must end with 'NEBU'.")]
    InvalidMintAddress,
    #[msg("Invalid tier specified.")]
    InvalidTier,
    #[msg("This tier is not eligible for rewards.")]
    TierNotEligible,
}
