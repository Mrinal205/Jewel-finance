import { PublicKey } from '@solana/web3.js';

export * from './jewel-bank';
export * from './jewel-farm';
export * from './jewel-common';

export const JEWEL_BANK_PROG_ID = new PublicKey(
  'bankHHdqMuaaST4qQk6mkzxGeKPHWmqdgor6Gs8r88m'
);
export const JEWEL_FARM_PROG_ID = new PublicKey(
  'farmL4xeBFVXJqtfxCzU9b28QACM7E2W2ctT6epAjvE'
);
