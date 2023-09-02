import { PublicKey } from '@solana/web3.js';
import { JEWEL_BANK_PROG_ID } from '../index';

export const findVaultPDA = async (bank: PublicKey, creator: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('vault'), bank.toBytes(), creator.toBytes()],
    JEWEL_BANK_PROG_ID
  );
};

export const findJewelBoxPDA = async (vault: PublicKey, mint: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('jewel_box'), vault.toBytes(), mint.toBytes()],
    JEWEL_BANK_PROG_ID
  );
};

export const findGdrPDA = async (vault: PublicKey, mint: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('jewel_deposit_receipt'), vault.toBytes(), mint.toBytes()],
    JEWEL_BANK_PROG_ID
  );
};

export const findVaultAuthorityPDA = async (vault: PublicKey) => {
  return PublicKey.findProgramAddress([vault.toBytes()], JEWEL_BANK_PROG_ID);
};

export const findWhitelistProofPDA = async (
  bank: PublicKey,
  whitelistedAddress: PublicKey
) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('whitelist'), bank.toBytes(), whitelistedAddress.toBytes()],
    JEWEL_BANK_PROG_ID
  );
};

export const findRarityPDA = async (bank: PublicKey, mint: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('jewel_rarity'), bank.toBytes(), mint.toBytes()],
    JEWEL_BANK_PROG_ID
  );
};
