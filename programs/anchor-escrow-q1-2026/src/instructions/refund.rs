use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint,
        TokenInterface,
        TokenAccount,
        TransferChecked,
        transfer_checked
    }
};
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    pub system_program: Program<'info,System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

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
        mut,
        close = maker,
        seeds = [
            b"escrow",
            maker.key().as_ref(),
            &escrow.seed.to_le_bytes()
            ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info,Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info,TokenAccount>,

}

impl<'info> Refund<'info> {
    pub fn refund(&mut self )-> Result<()>{

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_x.to_account_info(),
            to: self.maker_ata_x.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let salt_seed_bytes:&[u8] = &self.escrow.seed.to_le_bytes();
        let cpi_seed_signer:&[&[&[u8]]] = &[&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            salt_seed_bytes,
            &[self.escrow.bump]
        ]];


        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            cpi_seed_signer
        );

        transfer_checked(
            cpi_context,
            self.vault.amount,
            self.mint_x.decimals
        )
    }
}