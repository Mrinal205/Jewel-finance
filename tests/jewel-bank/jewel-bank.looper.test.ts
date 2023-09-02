import * as anchor from '@project-serum/anchor';
import { AnchorProvider, BN } from '@project-serum/anchor';
import { JewelBankClient, ITokenData, NodeWallet } from '../../src';
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { assert } from 'chai';

interface IJewel {
  jewel: ITokenData;
  jewelBox: PublicKey;
  jewelAmount: BN;
}

interface IVault {
  vault: PublicKey;
  vaultOwner: Keypair;
  vaultAuth: PublicKey;
  jewelBoxes: IJewel[];
}

/*
 * The purpose of this test is to:
 * 1) create A LOT of concurrent deposits -> make sure the program can handle
 * 2) test finding & deserializing appropriate PDA state accounts
 */
describe('looper', () => {
  const _provider = AnchorProvider.local();
  const gb = new JewelBankClient(
    _provider.connection,
    _provider.wallet as anchor.Wallet
  );
  const nw = new NodeWallet(
    _provider.connection,
    _provider.wallet as anchor.Wallet
  );

  const nVaults = 10;
  const nJewelsPerVault = 5;

  const bank = Keypair.generate();
  const bankManager = nw.wallet.publicKey;

  let vaults: IVault[] = [];

  async function prepVault() {
    const vaultOwner = await nw.createFundedWallet(100 * LAMPORTS_PER_SOL);

    const { vault, vaultAuth } = await gb.initVault(
      bank.publicKey,
      vaultOwner,
      vaultOwner,
      vaultOwner.publicKey,
      'test_vault'
    );

    vaults.push({
      vault,
      vaultOwner,
      vaultAuth,
      jewelBoxes: [],
    });
  }

  async function prepJewelDeposit(vault: IVault) {
    //many jewels, different amounts, but same owner (who also owns the vault)
    const { jewelAmount, jewel } = await prepJewel(vault.vaultOwner);

    const { jewelBox } = await gb.depositJewel(
      bank.publicKey,
      vault.vault,
      vault.vaultOwner,
      jewelAmount,
      jewel.tokenMint,
      jewel.tokenAcc
    );
    vault.jewelBoxes.push({
      jewel,
      jewelBox,
      jewelAmount,
    });
  }

  async function prepJewelWithdrawal(vault: IVault, jewelIdx: number) {
    const g = vault.jewelBoxes[jewelIdx];

    await gb.withdrawJewel(
      bank.publicKey,
      vault.vault,
      vault.vaultOwner,
      g.jewelAmount,
      g.jewel.tokenMint,
      vault.vaultOwner.publicKey //the receiver = owner of jewelDest, NOT jewelDest itself
    );
  }

  async function prepJewel(owner?: Keypair) {
    const jewelAmount = new BN(10); //here intentionally using 10
    const jewelOwner =
      owner ?? (await nw.createFundedWallet(100 * LAMPORTS_PER_SOL));
    const jewel = await nw.createMintAndFundATA(jewelOwner.publicKey, jewelAmount);

    return { jewelAmount, jewelOwner, jewel };
  }

  async function depositLooper() {
    //bank
    await gb.initBank(bank, bankManager, bankManager);
    console.log('bank started');

    //vaults
    const vaultPromises = [];
    for (let i = 0; i < nVaults; i++) {
      vaultPromises.push(prepVault());
    }
    await Promise.all(vaultPromises);
    console.log('vaults created');

    //jewels
    const jewelPromises: any[] = [];
    vaults.forEach((v: IVault) => {
      for (let i = 0; i < nJewelsPerVault; i++) {
        jewelPromises.push(prepJewelDeposit(v));
      }
    });
    await Promise.all(jewelPromises);
    console.log('jewels deposited');
  }

  async function withdrawalLooper() {
    const promises: any[] = [];
    vaults.forEach((v: IVault) => {
      for (let i = 0; i < nJewelsPerVault; i++) {
        promises.push(prepJewelWithdrawal(v, i));
      }
    });
    await Promise.all(promises);
    console.log('jewels withdrawn');
  }

  it('creates A LOT of PDAs & fetches them correctly', async () => {
    await depositLooper();

    // --------------------------------------- w/o constraints
    let bankPDAs = await gb.fetchAllBankPDAs();
    let vaultPDAs = await gb.fetchAllVaultPDAs();
    let gdrPDAs = await gb.fetchAllGdrPDAs();

    //verify correct # of accounts found
    assert.equal(bankPDAs.length, 1);
    assert.equal(vaultPDAs.length, nVaults);
    assert.equal(gdrPDAs.length, nVaults * nJewelsPerVault);

    //verify correct # of accounts stored
    let bankAcc = await gb.fetchBankAcc(bank.publicKey);
    assert(bankAcc.vaultCount.eq(new BN(nVaults)));

    for (const v of vaults) {
      const vaultAcc = await gb.fetchVaultAcc(v.vault);
      assert(vaultAcc.jewelBoxCount.eq(new BN(nJewelsPerVault)));
      assert(vaultAcc.jewelCount.eq(new BN(nJewelsPerVault).mul(new BN(10))));
      assert(vaultAcc.rarityPoints.eq(new BN(nJewelsPerVault).mul(new BN(10))));
    }

    // --------------------------------------- w/ constraints
    bankPDAs = await gb.fetchAllBankPDAs(bankManager);
    vaultPDAs = await gb.fetchAllVaultPDAs(bank.publicKey);

    //verify correct # of accounts found
    assert.equal(bankPDAs.length, 1);
    assert.equal(vaultPDAs.length, nVaults);

    for (const v of vaults) {
      const gdrPDAsByVault = await gb.fetchAllGdrPDAs(v.vault);
      assert.equal(gdrPDAsByVault.length, nJewelsPerVault);
    }
  });

  it('reduces PDA count after closure', async () => {
    await withdrawalLooper();

    const gdrPDAs = await gb.fetchAllGdrPDAs();

    //verify correct # of accounts found
    assert.equal(gdrPDAs.length, 0); //reduced after closure

    //verify correct # of accounts stored
    for (const v of vaults) {
      const vaultAcc = await gb.fetchVaultAcc(v.vault);
      assert(vaultAcc.jewelBoxCount.eq(new BN(0))); //reduced after closure
      assert(vaultAcc.jewelCount.eq(new BN(0)));
      assert(vaultAcc.rarityPoints.eq(new BN(0)));
    }
  });
});
