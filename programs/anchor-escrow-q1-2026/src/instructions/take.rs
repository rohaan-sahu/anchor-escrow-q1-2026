use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        TokenAccount,
        TokenInterface,
        Mint,
        TransferChecked,
        transfer_checked
    }
};

use crate::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    pub system_program: Program<'info,System>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub token_program: Interface<'info,TokenInterface>,

    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mint::token_program = token_program)]
    pub mint_x: InterfaceAccount<'info,Mint>,
    #[account(mint::token_program = token_program)]
    pub mint_y: InterfaceAccount<'info,Mint>,


	// Enclosing PDAs inside Box<>. Aparently the side of this instruct is large and 'Box<>' mitigates it.
	// My high-level  overview is that , 'Box<>' is a smart pointer, which can store teh programs somewhere
	//  else and simply point to then.
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = maker,						// This was incorrectly set to 'taker'
        associated_token::token_program = token_program
    )]
    pub maker_ata_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,                                  // Only 'taker' is the signer and the payer here. Putting 'maker' here is a likely candidate for causing "Access Violation" error.
        associated_token::mint = mint_x,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_x: Box<InterfaceAccount<'info,TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,                                  // Only 'taker' is the signer and the payer here. Putting 'maker' here is a likely candidate for causing "Access Violation" error.
        associated_token::mint = mint_y,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_y: Box<InterfaceAccount<'info,TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,                        // This was set to mint_x initially.
        associated_token::authority = taker,                    // This was set to maker. That might have caused "Access violation"
        associated_token::token_program = token_program
    )]
    pub taker_ata_y: Box<InterfaceAccount<'info,TokenAccount>>,


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
    pub escrow: Box<Account<'info,Escrow>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: Box<InterfaceAccount<'info,TokenAccount>>
}

impl<'info> Take<'info> {
    pub fn take(&mut self)-> Result<()>{
        let transfer_y_to_maker_accounts = TransferChecked {
            from: self.taker_ata_y.to_account_info(),
            mint: self.mint_y.to_account_info(),
            to: self.maker_ata_y.to_account_info(),
            authority: self.taker.to_account_info()
        };

        let transfer_x_to_taker_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_x.to_account_info(),
            to: self.taker_ata_x.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let cpi_context_y_to_maker = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_y_to_maker_accounts
        );

        transfer_checked(
            cpi_context_y_to_maker,
            self.escrow.receive,
            self.mint_y.decimals
        )?;

        // This holds the 'seed' evev after 'Escrow' closes.
        // 'bump' as well
        // Hence it might help avert the 'Access violation' error.
        let seed = self.escrow.seed;
        let bump = self.escrow.bump;

        let salt_seed_bytes:[u8;8] = seed.to_le_bytes();
        let cpi_seed_signer:&[&[&[u8]]] = &[&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &salt_seed_bytes,
            &[bump]
        ]];

        let cpi_context_x_to_taker = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_x_to_taker_accounts,
            cpi_seed_signer
        );

        transfer_checked(
            cpi_context_x_to_taker,
            self.vault.amount,
            self.mint_x.decimals
        )?;

        Ok(())
    }
}

// Saw this error while building
// Used 'claud' chat to find a solution
// Using Box was it's suggestion

// Error: Function _ZN149_$LT$anchor_escrow_q1_2026..instructions..
// take..Take$u20$as$u20$anchor_lang..Accounts$LT$anchor_escrow_q1_2026..
// instructions..take..TakeBumps$GT$$GT$12try_accounts17h345c22ed7bcc976eE 
// Stack offset of 4456 exceeded max offset of 4096 by 360 bytes,
// please minimize large stack variables.
// Estimated function frame size: 4800 bytes.
// Exceeding the maximum stack offset may cause undefined behavior during execution.
