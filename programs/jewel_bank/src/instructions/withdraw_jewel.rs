use crate::instructions::calc_rarity_points;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
use jewel_common::{errors::ErrorCode, *};

use crate::state::*;

#[derive(Accounts)]
#[instruction(bump_auth: u8, bump_jewel_box: u8, bump_gdr: u8, bump_rarity: u8)]
pub struct WithdrawJewel<'info> {
    // bank
    pub bank: Box<Account<'info, Bank>>,

    // vault
    // same rationale for not verifying the PDA as in deposit
    #[account(mut, has_one = bank, has_one = owner, has_one = authority)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(seeds = [vault.key().as_ref()], bump = bump_auth)]
    pub authority: AccountInfo<'info>,

    // jewel
    #[account(mut, seeds = [
            b"jewel_box".as_ref(),
            vault.key().as_ref(),
            jewel_mint.key().as_ref(),
        ],
        bump = bump_jewel_box)]
    pub jewel_box: Box<Account<'info, TokenAccount>>,
    #[account(mut, has_one = vault, has_one = jewel_mint, seeds = [
            b"jewel_deposit_receipt".as_ref(),
            vault.key().as_ref(),
            jewel_mint.key().as_ref(),
        ],
        bump = bump_gdr)]
    pub jewel_deposit_receipt: Box<Account<'info, JewelDepositReceipt>>,
    #[account(init_if_needed,
        associated_token::mint = jewel_mint,
        associated_token::authority = receiver,
        payer = owner)]
    pub jewel_destination: Box<Account<'info, TokenAccount>>,
    pub jewel_mint: Box<Account<'info, Mint>>,
    // we MUST ask for this PDA both during deposit and withdrawal for sec reasons, even if it's zero'ed
    /// CHECK:
    #[account(seeds = [
            b"jewel_rarity".as_ref(),
            bank.key().as_ref(),
            jewel_mint.key().as_ref()
        ], bump = bump_rarity)]
    pub jewel_rarity: AccountInfo<'info>,
    // unlike with deposits, the jewel can be sent out to anyone, not just the owner
    /// CHECK:
    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    // misc
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> WithdrawJewel<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.jewel_box.to_account_info(),
                to: self.jewel_destination.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.jewel_box.to_account_info(),
                destination: self.receiver.to_account_info(),
                authority: self.authority.clone(),
            },
        )
    }
}

pub fn handler(ctx: Context<WithdrawJewel>, amount: u64) -> Result<()> {
    // verify vault not suspended
    let bank = &*ctx.accounts.bank;
    let vault = &ctx.accounts.vault;

    if vault.access_suspended(bank.flags)? {
        return Err(error!(ErrorCode::VaultAccessSuspended));
    }

    // do the transfer
    token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&vault.vault_seeds()]),
        amount,
    )?;

    // update the gdr
    let gdr = &mut *ctx.accounts.jewel_deposit_receipt;
    let jewel_box = &ctx.accounts.jewel_box;

    gdr.jewel_count.try_sub_assign(amount)?;

    // this check is semi-useless but won't hurt
    if gdr.jewel_count != jewel_box.amount.try_sub(amount)? {
        return Err(error!(ErrorCode::AmountMismatch));
    }

    // if jewelbox empty, close both the box and the GDR, and return funds to user
    if gdr.jewel_count == 0 {
        // close jewel box
        token::close_account(
            ctx.accounts
                .close_context()
                .with_signer(&[&vault.vault_seeds()]),
        )?;

        // close GDR
        let receiver = &mut ctx.accounts.receiver;
        let gdr = &mut (*ctx.accounts.jewel_deposit_receipt).to_account_info();

        close_account(gdr, receiver)?;

        // decrement jewel box count stored in vault's state
        let vault = &mut ctx.accounts.vault;
        vault.jewel_box_count.try_sub_assign(1)?;
    }

    // decrement jewel count as well
    let vault = &mut ctx.accounts.vault;
    vault.jewel_count.try_sub_assign(amount)?;
    vault
        .rarity_points
        .try_sub_assign(calc_rarity_points(&ctx.accounts.jewel_rarity, amount)?)?;

    //msg!("{} jewels withdrawn from ${} jewel box", amount, jewel_box.key());
    Ok(())
}
