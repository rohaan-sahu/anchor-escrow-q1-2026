use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        AssociatedToken
    },
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface,
        TransferChecked,
        transfer_checked
    }
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    pub system_program: Program<'info,System>,
    pub token_program: Interface<'info,TokenInterface>,
    pub associated_token_program: Program<'info,AssociatedToken>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_x: InterfaceAccount<'info,Mint>,

    #[account(mint::token_program = token_program)]
    pub mint_y: InterfaceAccount<'info,Mint>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority  = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_x: InterfaceAccount<'info,TokenAccount>, 

    #[account(
        init,
        space = Escrow::DISCRIMINATOR.len() + Escrow::INIT_SPACE,
        payer = maker,
        seeds = [
            b"escrow",
            maker.key().as_ref(),
            seed.to_le_bytes().as_ref()
            ],
        bump,
    )]
    pub escrow: Account<'info,Escrow>,

    /// We can't have 'associated_token' constraint & 'seed' constarint together.
    /// This is prpbably a safety check, because an ATA is a PDA by design.
    /// Hence the setup below is wrong.
    // #[account(
    //     init,
    //     payer = maker,
    //     associated_token::mint = mint_x,
    //     associated_token::authority = escrow,
    //     associated_token::token_program = token_program,
    //     seeds = [
    //         b"vault",
    //         maker.key().as_ref(),
    //         escrow.key().as_ref(),
    //         ],
    //     bump
    // )]

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_x,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info,TokenAccount>,
}

impl<'info> Make<'info>{
    pub fn init_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps )-> Result<()> {
        self.escrow.set_inner(
            Escrow {
                seed,
                maker: self.maker.key(),
                mint_x: self.mint_x.key(),
                mint_y: self.mint_y.key(),
                receive,
                bump: bumps.escrow
            }
        );

        Ok(())
    }

    /*
    // Add additional fun to Escrow, to be able to create 'maker_ata_y' if it doesn't exist already
    // This may take the load off the 'taker' to crate teh ATA for the maker.
    pub fn ata_y_creation_fund(&mut self)-> Result<()>{

        // if (derived_ata.to_account_info().data_is_empty() == true){

        // }


        Ok(())
    }
    */

    pub fn deposit(&mut self, deposit: u64)-> Result<()> {
        let transfer_accounts = TransferChecked{
            from: self.maker_ata_x.to_account_info(),
            mint: self.mint_x.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info()
        };

        let cpi_context = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_accounts
        );

        transfer_checked(cpi_context,deposit,self.mint_x.decimals)
        
    }
}

