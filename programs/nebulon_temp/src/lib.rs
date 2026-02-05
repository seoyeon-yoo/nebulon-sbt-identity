use anchor_lang::prelude::*;

declare_id!("Hsb6bsCkWVg6nMh9MjxvyCWaEm14kEW7yRPEqgTttQhz");

#[program]
pub mod nebulon_temp {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
