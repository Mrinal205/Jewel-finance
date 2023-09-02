use anchor_lang::{prelude::*, Discriminator};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use arrayref::array_ref;
use jewel_common::{errors::ErrorCode, *};

use crate::{assert_decode_metadata, state::*};

#[derive(Accounts)]
#[instruction(bump_auth: u8, bump_rarity: u8)]
pub struct DepositJewel<'info> {
    // bank
    pub bank: Box<Account<'info, Bank>>,

    // vault
    // skipped vault PDA verification because requires passing in creator, which is tedious
    // sec wise secure enough: vault has owner -> owner is signer
    #[account(mut, has_one = bank, has_one = owner, has_one = authority)]
    pub vault: Box<Account<'info, Vault>>,
    // currently only the vault owner can deposit
    // add a "depositor" account, and remove Signer from vault owner to let anyone to deposit
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(seeds = [vault.key().as_ref()], bump = bump_auth)]
    pub authority: AccountInfo<'info>,

    // jewel
    #[account(init_if_needed, seeds = [
            b"jewel_box".as_ref(),
            vault.key().as_ref(),
            jewel_mint.key().as_ref(),
        ],
        bump,
        token::mint = jewel_mint,
        token::authority = authority,
        payer = owner)]
    pub jewel_box: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, seeds = [
            b"jewel_deposit_receipt".as_ref(),
            vault.key().as_ref(),
            jewel_mint.key().as_ref(),
        ],
        bump,
        payer = owner,
        space = 8 + std::mem::size_of::<JewelDepositReceipt>())]
    pub jewel_deposit_receipt: Box<Account<'info, JewelDepositReceipt>>,
    #[account(mut)]
    pub jewel_source: Box<Account<'info, TokenAccount>>,
    pub jewel_mint: Box<Account<'info, Mint>>,
    // we MUST ask for this PDA both during deposit and withdrawal for sec reasons, even if it's zero'ed
    /// CHECK:
    #[account(seeds = [
            b"jewel_rarity".as_ref(),
            bank.key().as_ref(),
            jewel_mint.key().as_ref()
        ],
        bump = bump_rarity)]
    pub jewel_rarity: AccountInfo<'info>,

    // misc
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    //
    // remaining accounts could be passed, in this order:
    // - mint_whitelist_proof
    // - jewel_metadata <- if we got to this point we can assume jewel = NFT, not a fungible token
    // - creator_whitelist_proof
}

impl<'info> DepositJewel<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.jewel_source.to_account_info(),
                to: self.jewel_box.to_account_info(),
                authority: self.owner.to_account_info(),
            },
        )
    }
}

fn assert_valid_whitelist_proof<'info>(
    whitelist_proof: &AccountInfo<'info>,
    bank: &Pubkey,
    address_to_whitelist: &Pubkey,
    program_id: &Pubkey,
    expected_whitelist_type: WhitelistType,
) -> Result<()> {
    // 1 verify the PDA seeds match
    let seed = &[
        b"whitelist".as_ref(),
        bank.as_ref(),
        address_to_whitelist.as_ref(),
    ];
    let (whitelist_addr, _bump) = Pubkey::find_program_address(seed, program_id);

    // we can't use an assert_eq statement, we want to catch this error and continue along to creator testing
    if whitelist_addr != whitelist_proof.key() {
        return Err(error!(ErrorCode::NotWhitelisted));
    }

    // 2 no need to verify ownership, deserialization does that for us
    // https://github.com/project-serum/anchor/blob/fcb07eb8c3c9355f3cabc00afa4faa6247ccc960/lang/src/account.rs#L36
    let proof = Account::<'info, WhitelistProof>::try_from(whitelist_proof)?;

    // 3 verify whitelist type matches
    proof.contains_type(expected_whitelist_type)
}

fn assert_whitelisted<'info>(ctx: &Context<'_, '_, '_, 'info, DepositJewel<'info>>) -> Result<()> {
    let bank = &*ctx.accounts.bank;
    let mint = &*ctx.accounts.jewel_mint;
    let remaining_accs = &mut ctx.remaining_accounts.iter();

    // whitelisted mint is always the 1st optional account
    // this is because it's applicable to both NFTs and standard fungible tokens
    let mint_whitelist_proof_info = next_account_info(remaining_accs)?;

    // attempt to verify based on mint
    if bank.whitelisted_mints > 0 {
        if let Ok(()) = assert_valid_whitelist_proof(
            mint_whitelist_proof_info,
            &bank.key(),
            &mint.key(),
            ctx.program_id,
            WhitelistType::MINT,
        ) {
            // msg!("mint whitelisted: {}, going ahead", &mint.key());
            return Ok(());
        }
    }

    // if mint verification above failed, attempt to verify based on creator
    if bank.whitelisted_creators > 0 {
        // 2 additional accounts are expected - metadata and creator whitelist proof
        let metadata_info = next_account_info(remaining_accs)?;
        let creator_whitelist_proof_info = next_account_info(remaining_accs)?;

        // verify metadata is legit
        let metadata = assert_decode_metadata(&mint, &metadata_info)?;

        // metaplex constraints this to max 5, so won't go crazy on compute
        // (empirical testing showed there's practically 0 diff between stopping at 0th and 5th creator)
        for creator in &metadata.data.creators.unwrap() {
            // verify creator actually signed off on this nft
            if !creator.verified {
                continue;
            }

            // check if creator is whitelisted, returns an error if not
            let attempted_proof = assert_valid_whitelist_proof(
                creator_whitelist_proof_info,
                &bank.key(),
                &creator.address,
                ctx.program_id,
                WhitelistType::CREATOR,
            );

            match attempted_proof {
                //proof succeeded, return out of the function, no need to continue looping
                Ok(()) => return Ok(()),
                //proof failed, continue to check next creator
                Err(_e) => continue,
            }
        }
    }

    // if both conditions above failed tok return Ok(()), then verification failed
    Err(error!(ErrorCode::NotWhitelisted))
}

/// if rarity account is present, extract rarities from there - else use 1 * amount
pub fn calc_rarity_points(jewel_rarity: &AccountInfo, amount: u64) -> Result<u64> {
    if !jewel_rarity.data_is_empty() {
        let rarity_account = Account::<Rarity>::try_from(jewel_rarity)?;
        amount.try_mul(rarity_account.points as u64)
    } else {
        Ok(amount)
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DepositJewel<'info>>,
    amount: u64,
) -> Result<()> {
    // fix missing discriminator check
    {
        let acct = ctx.accounts.jewel_deposit_receipt.to_account_info();
        let data: &[u8] = &acct.try_borrow_data()?;
        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &JewelDepositReceipt::discriminator() && disc_bytes.iter().any(|a| a != &0) {
            return Err(error!(ErrorCode::AccountDiscriminatorMismatch));
        }
    }

    // if even a single whitelist exists, verify the token against it
    let bank = &*ctx.accounts.bank;

    if bank.whitelisted_mints > 0 || bank.whitelisted_creators > 0 {
        assert_whitelisted(&ctx)?;
    }

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

    // record total number of jewel boxes in vault's state
    let vault = &mut ctx.accounts.vault;
    vault.jewel_box_count.try_add_assign(1)?;
    vault.jewel_count.try_add_assign(amount)?;
    vault
        .rarity_points
        .try_add_assign(calc_rarity_points(&ctx.accounts.jewel_rarity, amount)?)?;

    // record a gdr
    let gdr = &mut *ctx.accounts.jewel_deposit_receipt;
    let jewel_box = &*ctx.accounts.jewel_box;

    gdr.vault = vault.key();
    gdr.jewel_box_address = jewel_box.key();
    gdr.jewel_mint = jewel_box.mint;
    gdr.jewel_count.try_add_assign(amount)?;

    // this check is semi-useless but won't hurt
    if gdr.jewel_count != jewel_box.amount.try_add(amount)? {
        // msg!("{} {}", gdr.jewel_count, jewel_box.amount);
        return Err(error!(ErrorCode::AmountMismatch));
    }

    // msg!("{} jewels deposited into {} jewel box", amount, jewel_box.key());
    Ok(())
}
