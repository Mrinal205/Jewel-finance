import { PublicKey } from '@solana/web3.js';
import { JEWEL_FARM_PROG_ID } from '../index';

export const findFarmerPDA = async (farm: PublicKey, identity: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('farmer'), farm.toBytes(), identity.toBytes()],
    JEWEL_FARM_PROG_ID
  );
};

export const findFarmAuthorityPDA = async (farm: PublicKey) => {
  return PublicKey.findProgramAddress([farm.toBytes()], JEWEL_FARM_PROG_ID);
};

export const findFarmTreasuryPDA = (farm: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('treasury'), farm.toBytes()],
    JEWEL_FARM_PROG_ID
  );
};

export const findAuthorizationProofPDA = (
  farm: PublicKey,
  funder: PublicKey
) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('authorization'), farm.toBytes(), funder.toBytes()],
    JEWEL_FARM_PROG_ID
  );
};

export const findRewardsPotPDA = (farm: PublicKey, rewardMint: PublicKey) => {
  return PublicKey.findProgramAddress(
    [Buffer.from('reward_pot'), farm.toBytes(), rewardMint.toBytes()],
    JEWEL_FARM_PROG_ID
  );
};
