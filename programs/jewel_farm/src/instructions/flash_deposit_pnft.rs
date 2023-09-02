use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use jewel_bank::{
    self,
    cpi::accounts::{DepositJewelPnft, ProgNftShared, SetVaultLock},
    instructions::calc_rarity_points,
    program::JewelBank,
    state::{Bank, Vault},
};
use jewel_common::*;

use crate::{instructions::FEE_WALLET, state::*};

const FEE_LAMPORTS: u64 = 2_000_000; // 0.002 SOL per stake/unstake
const FD_FEE_LAMPORTS: u64 = 1_000_000; // half of that for FDs

#[derive(Accounts)]
#[instruction(bump_farmer: u8)]
pub struct FlashDepositPnft<'info> {
    // farm
    #[account(mut, has_one = farm_authority)]
    pub farm: Box<Account<'info, Farm>>,
    //skipping seeds verification to save compute budget, has_one check above should be enough
    /// CHECK:
    pub farm_authority: AccountInfo<'info>,

    // farmer
    #[account(mut, has_one = farm, has_one = identity, has_one = vault,
        seeds = [
            b"farmer".as_ref(),
            farm.key().as_ref(),
            identity.key().as_ref(),
        ],
        bump = bump_farmer)]
    pub farmer: Box<Account<'info, Farmer>>,
    #[account(mut)]
    pub identity: Signer<'info>,

    // cpi
    pub bank: Box<Account<'info, Bank>>,
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK:
    pub vault_authority: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(mut)]
    pub jewel_box: AccountInfo<'info>,
    // trying to deserialize here leads to errors (doesn't exist yet)
    /// CHECK:
    #[account(mut)]
    pub jewel_deposit_receipt: AccountInfo<'info>,
    #[account(mut)]
    pub jewel_source: Box<Account<'info, TokenAccount>>,
    pub jewel_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    pub jewel_rarity: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub jewel_bank: Program<'info, JewelBank>,
    /// CHECK:
    #[account(mut, address = Pubkey::from_str(FEE_WALLET).unwrap())]
    pub fee_acc: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    ///CHECK: downstream
    #[account(mut)]
    pub jewel_metadata: UncheckedAccount<'info>,
    ///CHECK: downstream
    pub jewel_edition: UncheckedAccount<'info>,
    ///CHECK: downstream
    #[account(mut)]
    pub owner_token_record: UncheckedAccount<'info>,
    ///CHECK: downstream
    #[account(mut)]
    pub dest_token_record: UncheckedAccount<'info>,
    ///CHECK: downstream
    pub token_metadata_program: UncheckedAccount<'info>,
    ///CHECK: downstream
    pub instructions: UncheckedAccount<'info>,
    ///CHECK: downstream
    pub authorization_rules_program: UncheckedAccount<'info>,
    //
    // remaining accounts could be passed, in this order:
    // - rules account
    // - mint_whitelist_proof
    // - creator_whitelist_proof
}

impl<'info> FlashDepositPnft<'info> {
    fn set_lock_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, SetVaultLock<'info>> {
        CpiContext::new(
            self.jewel_bank.to_account_info(),
            SetVaultLock {
                bank: self.bank.to_account_info(),
                vault: self.vault.to_account_info(),
                bank_manager: self.farm_authority.clone(),
            },
        )
    }

    fn deposit_jewel_ctx(&self) -> CpiContext<'_, '_, '_, 'info, DepositJewelPnft<'info>> {
        CpiContext::new(
            self.jewel_bank.to_account_info(),
            DepositJewelPnft {
                bank: self.bank.to_account_info(),
                vault: self.vault.to_account_info(),
                owner: self.identity.to_account_info(),
                authority: self.vault_authority.clone(),
                jewel_box: self.jewel_box.clone(),
                jewel_deposit_receipt: self.jewel_deposit_receipt.clone(),
                jewel_source: self.jewel_source.to_account_info(),
                jewel_mint: self.jewel_mint.to_account_info(),
                jewel_rarity: self.jewel_rarity.clone(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
                jewel_metadata: self.jewel_metadata.to_account_info(),
                jewel_edition: self.jewel_edition.to_account_info(),
                owner_token_record: self.owner_token_record.to_account_info(),
                dest_token_record: self.dest_token_record.to_account_info(),
                pnft_shared: ProgNftShared {
                    token_metadata_program: self.token_metadata_program.to_account_info(),
                    instructions: self.instructions.to_account_info(),
                    authorization_rules_program: self.authorization_rules_program.to_account_info(),
                },
            },
        )
    }

    fn transfer_fee(&self, fee: u64) -> Result<()> {
        invoke(
            &system_instruction::transfer(self.identity.key, self.fee_acc.key, fee),
            &[
                self.identity.to_account_info(),
                self.fee_acc.clone(),
                self.system_program.to_account_info(),
            ],
        )
        .map_err(Into::into)
    }
}

pub fn handler<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, FlashDepositPnft<'info>>,
    bump_vault_auth: u8,
    bump_rarity: u8,
    amount: u64,
    rules_acc_present: bool,
) -> Result<()> {
    // flash deposit a jewel into a locked vault
    jewel_bank::cpi::set_vault_lock(
        ctx.accounts
            .set_lock_vault_ctx()
            .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
        false,
    )?;

    jewel_bank::cpi::deposit_jewel_pnft(
        ctx.accounts
            .deposit_jewel_ctx()
            .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        bump_vault_auth,
        bump_rarity,
        amount,
        None, //fuck this
        rules_acc_present,
    )?;

    jewel_bank::cpi::set_vault_lock(
        ctx.accounts
            .set_lock_vault_ctx()
            .with_signer(&[&ctx.accounts.farm.farm_seeds()]),
        true,
    )?;

    // update accrued rewards BEFORE we increment the stake
    let farm = &mut ctx.accounts.farm;
    let farmer = &mut ctx.accounts.farmer;
    let now_ts = now_ts()?;

    farm.update_rewards(now_ts, Some(farmer), true)?;

    ctx.accounts.vault.reload()?;

    // in case the command is used BEFORE farmer staked
    if farmer.jewels_staked == 0 {
        farm.begin_staking(
            now_ts,
            ctx.accounts.vault.jewel_count,
            ctx.accounts.vault.rarity_points,
            farmer,
        )?;
        //collect a fee for staking
        ctx.accounts.transfer_fee(FEE_LAMPORTS)?;
    } else {
        let extra_rarity = calc_rarity_points(&ctx.accounts.jewel_rarity, amount)?;
        farm.stake_extra_jewels(
            now_ts,
            ctx.accounts.vault.jewel_count,
            ctx.accounts.vault.rarity_points,
            amount,
            extra_rarity,
            farmer,
        )?;
        //collect a fee for staking
        ctx.accounts.transfer_fee(FD_FEE_LAMPORTS)?;
    }

    // msg!("{} extra jewels staked for {}", amount, farmer.key());
    Ok(())
}
