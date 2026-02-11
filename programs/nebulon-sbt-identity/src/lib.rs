use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, Token2022};
use anchor_spl::token_interface::{Mint as MintInterface, TokenAccount as TokenAccountInterface};
use anchor_lang::solana_program::system_instruction;
use std::collections::BTreeMap;

declare_id!("8AWzFHnngTCJQTGEQC2M1VHLEa52tXAkXKjzTMY2oxD1");

#[program]
pub mod nebulon_sbt_identity {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_mint: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        state.owner = ctx.accounts.admin.key();
        state.admins = vec![ctx.accounts.admin.key()];
        state.reward_mint = reward_mint;
        state.total_agents = 0;
        state.total_score = 0;
        state.vault_bump = ctx.bumps.reward_vault;
        state.state_bump = ctx.bumps.global_state;
        state.reward_pool = 0;
        Ok(())
    }

    pub fn add_admin(ctx: Context<ManageAdmins>, new_admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        
        // Verify owner authorization
        require_keys_eq!(ctx.accounts.owner.key(), state.owner, ErrorCode::Unauthorized);
        
        // Check admin limit
        require!(state.admins.len() < 10, ErrorCode::AdminLimitReached);
        
        // Check if admin already exists
        require!(!state.admins.contains(&new_admin), ErrorCode::AdminAlreadyExists);
        
        state.admins.push(new_admin);
        msg!("Added new admin: {}", new_admin);
        Ok(())
    }

    pub fn remove_admin(ctx: Context<ManageAdmins>, admin_to_remove: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        
        // Verify owner authorization
        require_keys_eq!(ctx.accounts.owner.key(), state.owner, ErrorCode::Unauthorized);
        
        // Cannot remove owner from admin list
        require_keys_neq!(admin_to_remove, state.owner, ErrorCode::CannotRemoveOwner);
        
        // Find and remove the admin
        let initial_len = state.admins.len();
        state.admins.retain(|a| *a != admin_to_remove);
        
        require!(state.admins.len() < initial_len, ErrorCode::AdminNotFound);
        
        msg!("Removed admin: {}", admin_to_remove);
        Ok(())
    }

    pub fn issue_identity(ctx: Context<IssueIdentity>, handle: String, name: String, uri: String, hex_id: [u8; 512]) -> Result<()> {
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
        
        // Read total_agents before mutable borrow
        let total_agents = ctx.accounts.global_state.total_agents;
        let calculated_fee = base_fee.saturating_add(increment.saturating_mul(total_agents));
        let fee_amount = std::cmp::min(calculated_fee, max_fee);

        // Get keys before any mutable borrows
        let owner_key = ctx.accounts.owner.key();
        let global_state_key = ctx.accounts.global_state.key();

        // Transfer SOL to global_state PDA instead of admin directly
        let ix = system_instruction::transfer(
            &owner_key,
            &global_state_key,
            fee_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.global_state.to_account_info(),
            ],
        )?;

        // Now do mutable borrows
        let identity = &mut ctx.accounts.identity;
        identity.owner = owner_key;
        identity.mint = ctx.accounts.sbt_mint.key();
        identity.handle = handle.clone();
        identity.hex_id = hex_id;
        identity.score = 0;
        identity.is_active = true;
        identity.uri = uri;
        identity.last_claim_ts = Clock::get()?.unix_timestamp;
        identity.sns = BTreeMap::new();
        identity.private_vault = Vec::new();
        identity.tier = 10; 
        identity.recommendations = 0;
        identity.reports = 0;

        let state = &mut ctx.accounts.global_state;
        state.total_agents += 1;

        msg!("Identity issued for agent: {} with handle {}", name, handle);
        Ok(())
    }

    /// Update Agent Score, Tier, and Metadata URI (Dynamic NFT Identity)
    pub fn update_agent_status(ctx: Context<UpdateAgentStatus>, new_score: u64, tier: u8, new_uri: String) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        let state = &mut ctx.accounts.global_state;
        
        // Check if caller is in admins list
        require!(state.admins.contains(&ctx.accounts.admin.key()), ErrorCode::Unauthorized);
        require!(tier >= 1 && tier <= 10, ErrorCode::InvalidTier);

        state.total_score = state.total_score.saturating_sub(identity.score).saturating_add(new_score);
        identity.score = new_score;
        identity.tier = tier;
        identity.uri = new_uri; // Dynamic URI update based on tier
        
        msg!("Status updated for {}: Score {}, Tier {}, URI {}", identity.handle, new_score, tier, identity.uri);
        Ok(())
    }

    pub fn update_sns(ctx: Context<UpdateSns>, platform: String, handle: String, remove: bool) -> Result<()> {
        let state = &ctx.accounts.global_state;
        
        // Check if caller is in admins list
        require!(state.admins.contains(&ctx.accounts.admin.key()), ErrorCode::Unauthorized);

        let identity = &mut ctx.accounts.identity;
        if remove {
            identity.sns.remove(&platform);
        } else {
            identity.sns.insert(platform, handle);
        }
        Ok(())
    }

    pub fn update_private_vault(ctx: Context<UpdateAgentData>, encrypted_data: Vec<u8>) -> Result<()> {
        let identity = &mut ctx.accounts.identity;
        require_keys_eq!(ctx.accounts.owner.key(), identity.owner, ErrorCode::Unauthorized);
        identity.private_vault = encrypted_data;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let state = &ctx.accounts.global_state;
        require!(identity.is_active, ErrorCode::InactiveIdentity);
        require!(identity.tier < 10, ErrorCode::TierNotEligible);

        let reward_amount = 1000; 
        
        let seeds = &[b"reward_vault", state.reward_mint.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        anchor_spl::token_interface::transfer_checked(ctx.accounts.into_transfer_context().with_signer(signer), reward_amount, ctx.accounts.reward_mint.decimals)?;
        Ok(())
    }

    /// Admin withdraws SOL from the global_state PDA
    pub fn admin_withdraw_sol(ctx: Context<AdminWithdrawSol>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.global_state;
        
        // Verify admin authorization - check if caller is in admins list
        require!(state.admins.contains(&ctx.accounts.admin.key()), ErrorCode::Unauthorized);
        
        // Check sufficient balance (account for rent-exempt minimum)
        let global_state_info = ctx.accounts.global_state.to_account_info();
        let rent = Rent::get()?;
        let min_balance = rent.minimum_balance(global_state_info.data_len());
        let available_balance = global_state_info.lamports().saturating_sub(min_balance);
        
        require!(amount <= available_balance, ErrorCode::InsufficientBalance);
        
        // Transfer SOL from global_state PDA to admin
        **global_state_info.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;
        
        msg!("Admin withdrew {} lamports from global_state", amount);
        Ok(())
    }

    /// Admin withdraws reward tokens (NEBU) from the reward_vault
    pub fn admin_withdraw_rewards(ctx: Context<AdminWithdrawRewards>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.global_state;
        
        // Verify admin authorization - check if caller is in admins list
        require!(state.admins.contains(&ctx.accounts.admin.key()), ErrorCode::Unauthorized);
        
        // Check sufficient token balance
        require!(ctx.accounts.reward_vault.amount >= amount, ErrorCode::InsufficientTokenBalance);
        
        // Transfer tokens from reward_vault to admin's token account
        let reward_mint_key = state.reward_mint;
        let seeds = &[b"reward_vault", reward_mint_key.as_ref(), &[state.vault_bump]];
        let signer = &[&seeds[..]];
        
        anchor_spl::token_interface::transfer_checked(
            ctx.accounts.into_transfer_context().with_signer(signer),
            amount,
            ctx.accounts.reward_mint.decimals,
        )?;
        
        msg!("Admin withdrew {} reward tokens from vault", amount);
        Ok(())
    }
    pub fn recommend_with_handle(ctx: Context<ActionAgent>, handle: String) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        require!(identity.handle == handle, ErrorCode::HandleMismatch);
        process_recommendation(ctx)
    }

    pub fn recommend_with_hex_id(ctx: Context<ActionAgent>, hex_id: [u8; 512]) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        // Constant-time comparison or simple loop to avoid DefaultHasher issues in SBF
        let mut match_found = true;
        for i in 0..512 {
            if identity.hex_id[i] != hex_id[i] {
                match_found = false;
                break;
            }
        }
        require!(match_found, ErrorCode::HexIdMismatch);
        process_recommendation(ctx)
    }

    pub fn recommend_with_sns(ctx: Context<ActionAgent>, platform: String, sns_handle: String) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        let stored_handle = identity.sns.get(&platform).ok_or(ErrorCode::SnsNotFound)?;
        require!(stored_handle == &sns_handle, ErrorCode::SnsHandleMismatch);
        process_recommendation(ctx)
    }

    pub fn report_with_handle(ctx: Context<ActionAgent>, handle: String) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        require!(identity.handle == handle, ErrorCode::HandleMismatch);
        process_report(ctx)
    }

    pub fn report_with_hex_id(ctx: Context<ActionAgent>, hex_id: [u8; 512]) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        let mut match_found = true;
        for i in 0..512 {
            if identity.hex_id[i] != hex_id[i] {
                match_found = false;
                break;
            }
        }
        require!(match_found, ErrorCode::HexIdMismatch);
        process_report(ctx)
    }

    pub fn report_with_sns(ctx: Context<ActionAgent>, platform: String, sns_handle: String) -> Result<()> {
        let identity = &mut ctx.accounts.target_identity;
        let stored_handle = identity.sns.get(&platform).ok_or(ErrorCode::SnsNotFound)?;
        require!(stored_handle == &sns_handle, ErrorCode::SnsHandleMismatch);
        process_report(ctx)
    }

    pub fn get_hex_id_by_sns(ctx: Context<GetHexIdBySns>, platform: String, handle: String) -> Result<()> {
        let identity = &ctx.accounts.identity;
        
        let stored_handle = identity.sns.get(&platform).ok_or(ErrorCode::SnsNotFound)?;
        require!(stored_handle == &handle, ErrorCode::SnsHandleMismatch);
        
        msg!("Hex ID for SNS {}:{}: {:?}", platform, handle, identity.hex_id);
        Ok(())
    }
}

// Internal helper functions for cleaner logic
fn process_recommendation(ctx: Context<ActionAgent>) -> Result<()> {
    let amount = 100_000_000; // 0.1 NEBU
    anchor_spl::token_interface::transfer_checked(
        ctx.accounts.into_transfer_context(),
        amount,
        ctx.accounts.reward_mint.decimals,
    )?;

    let identity = &mut ctx.accounts.target_identity;
    identity.recommendations = identity.recommendations.saturating_add(1);
    identity.score = identity.score.saturating_add(100);
    msg!("Agent {} recommended.", identity.handle);
    Ok(())
}

fn process_report(ctx: Context<ActionAgent>) -> Result<()> {
    let amount = 50_000_000; // 0.05 NEBU
    anchor_spl::token_interface::transfer_checked(
        ctx.accounts.into_transfer_context(),
        amount,
        ctx.accounts.reward_mint.decimals,
    )?;

    let identity = &mut ctx.accounts.target_identity;
    identity.reports = identity.reports.saturating_add(1);
    identity.score = identity.score.saturating_sub(200);
    msg!("Agent {} reported.", identity.handle);
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + (4 + 32 * 10) + 32 + 8 + 8 + 8 + 1 + 1, seeds = [b"global_state"], bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    #[account(init, payer = admin, token::mint = reward_mint, token::authority = reward_vault, seeds = [b"reward_vault", reward_mint.key().as_ref()], bump)]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

/// Context for owner to manage admins (add/remove)
#[derive(Accounts)]
pub struct ManageAdmins<'info> {
    #[account(mut, seeds = [b"global_state"], bump = global_state.state_bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(handle: String)]
pub struct IssueIdentity<'info> {
    #[account(mut, seeds = [b"global_state"], bump = global_state.state_bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + (4 + 32) + 512 + 8 + 1 + 200 + 8 + 500 + 500 + 1000 + 1 + 16, 
        seeds = [b"identity", handle.as_bytes()], 
        bump
    )]
    pub identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub owner: Signer<'info>,
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

/// Context for admin to withdraw SOL from global_state PDA
#[derive(Accounts)]
pub struct AdminWithdrawSol<'info> {
    #[account(mut, seeds = [b"global_state"], bump = global_state.state_bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Context for admin to withdraw reward tokens from reward_vault
#[derive(Accounts)]
pub struct AdminWithdrawRewards<'info> {
    #[account(seeds = [b"global_state"], bump = global_state.state_bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"reward_vault", reward_mint.key().as_ref()],
        bump = global_state.vault_bump
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    #[account(mut)]
    pub admin_token_account: InterfaceAccount<'info, TokenAccountInterface>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> AdminWithdrawRewards<'info> {
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

#[account]
pub struct GlobalState {
    pub owner: Pubkey,
    pub admins: Vec<Pubkey>,
    pub reward_mint: Pubkey,
    pub total_agents: u64,
    pub total_score: u64,
    pub reward_pool: u64,
    pub vault_bump: u8,
    pub state_bump: u8,
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
    pub private_vault: Vec<u8>,
    pub tier: u8, 
    pub recommendations: u64,
    pub reports: u64,
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
    #[msg("Insufficient SOL balance in global_state.")]
    InsufficientBalance,
    #[msg("Insufficient token balance in reward vault.")]
    InsufficientTokenBalance,
    #[msg("Maximum number of admins (10) has been reached.")]
    AdminLimitReached,
    #[msg("This admin already exists.")]
    AdminAlreadyExists,
    #[msg("Admin not found in the list.")]
    AdminNotFound,
    #[msg("Cannot remove the owner from the admin list.")]
    CannotRemoveOwner,
    #[msg("Insufficient NEBU for this action.")]
    InsufficientFunds,
    #[msg("SNS record not found for this platform.")]
    SnsNotFound,
    #[msg("SNS handle mismatch.")]
    SnsHandleMismatch,
    #[msg("Handle mismatch.")]
    HandleMismatch,
    #[msg("Hex ID mismatch.")]
    HexIdMismatch,
    #[msg("At least one identification method must be provided.")]
    NoIdentificationProvided,
}

#[derive(Accounts)]
pub struct GetHexIdBySns<'info> {
    pub identity: Account<'info, AgentIdentity>,
}

#[derive(Accounts)]
pub struct ActionAgent<'info> {
    #[account(mut, seeds = [b"global_state"], bump = global_state.state_bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub target_identity: Account<'info, AgentIdentity>,
    #[account(mut)]
    pub actor: Signer<'info>,
    #[account(mut)]
    pub actor_token_account: InterfaceAccount<'info, TokenAccountInterface>,
    #[account(
        mut,
        seeds = [b"reward_vault", reward_mint.key().as_ref()],
        bump = global_state.vault_bump
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccountInterface>,
    pub reward_mint: InterfaceAccount<'info, MintInterface>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> ActionAgent<'info> {
    fn into_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, anchor_spl::token_interface::TransferChecked<'info>> {
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
            from: self.actor_token_account.to_account_info(),
            to: self.reward_vault.to_account_info(),
            authority: self.actor.to_account_info(),
            mint: self.reward_mint.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
