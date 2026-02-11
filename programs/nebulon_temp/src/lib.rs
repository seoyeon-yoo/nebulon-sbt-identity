use anchor_lang::prelude::*;

declare_id!("HurJWpsGH7oqbm9Z22iuy8ymVy2EKazGzCnqg8DvEkco");

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
