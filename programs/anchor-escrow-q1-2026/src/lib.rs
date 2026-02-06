pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;



declare_id!("EgDVo2wVRW1LGSbBJBUtCEvswBnU3v6inYJcNwE34B8x");

#[program]
pub mod anchor_escrow_q1_2026 {
    use super::*;

    pub fn make(ctx: Context<Make>,seed: u64, deposit: u64, receive: u64) -> Result<()> {
        
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;

        ctx.accounts.deposit(deposit)?;

        Ok(())
    }

    pub fn refund(ctx: Context<Refund>,seed: u64) -> Result<()> {

        ctx.accounts.refund(seed)?;

        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()>{

        ctx.accounts.take()?;

        Ok(())
    }
}
