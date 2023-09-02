import chai, { assert, expect } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { defaultFarmConfig, JewelFarmTester } from '../jewel-farm.tester';
import { RewardType } from '../../../src';

chai.use(chaiAsPromised);

describe('misc', () => {
  let gf = new JewelFarmTester();

  beforeEach('preps accs', async () => {
    await gf.prepAccounts(45000);
  });

  it('exceeds max number of farmers', async () => {
    //farm
    const maxCounts = {
      maxFarmers: 1,
      maxJewels: 0,
      maxRarityPoints: 0,
    };
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed, maxCounts);

    //farmers
    await gf.callInitFarmer(gf.farmer1Identity);
    await gf.callInitFarmer(gf.farmer2Identity);
    await gf.callDeposit(gf.jewel1Amount, gf.farmer1Identity);
    await gf.callDeposit(gf.jewel2Amount, gf.farmer2Identity);

    //try staking
    await gf.stakeAndVerify(gf.farmer1Identity);
    await expect(gf.stakeAndVerify(gf.farmer2Identity)).to.be.rejectedWith(
      'TooManyFarmersStaked'
    );
  });

  it('exceeds max number of jewels', async () => {
    //farm
    const maxCounts = {
      maxFarmers: 0,
      maxJewels: gf.jewel1Amount.toNumber(),
      maxRarityPoints: 0,
    };
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed, maxCounts);

    //farmers
    await gf.callInitFarmer(gf.farmer1Identity);
    await gf.callInitFarmer(gf.farmer2Identity);
    await gf.callDeposit(gf.jewel1Amount, gf.farmer1Identity);
    await gf.callDeposit(gf.jewel2Amount, gf.farmer2Identity);

    //try staking
    await gf.stakeAndVerify(gf.farmer1Identity);
    await expect(gf.stakeAndVerify(gf.farmer2Identity)).to.be.rejectedWith(
      'TooManyJewelsStaked'
    );
  });

  it('exceeds max number of rarities', async () => {
    const maxCounts = {
      maxFarmers: 0,
      maxJewels: 0,
      maxRarityPoints: gf.jewel1Amount.toNumber(),
    };
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed, maxCounts);

    //farmers
    await gf.callInitFarmer(gf.farmer1Identity);
    await gf.callInitFarmer(gf.farmer2Identity);
    await gf.callDeposit(gf.jewel1Amount, gf.farmer1Identity);
    await gf.callDeposit(gf.jewel2Amount, gf.farmer2Identity);

    //try staking
    await gf.stakeAndVerify(gf.farmer1Identity);
    await expect(gf.stakeAndVerify(gf.farmer2Identity)).to.be.rejectedWith(
      'TooManyRarityPointsStaked'
    );
  });

  it('empty max counts - does nothing', async () => {
    //farm
    const maxCounts = {
      maxFarmers: 0,
      maxJewels: 0,
      maxRarityPoints: 0,
    };
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed, maxCounts);

    //farmers
    await gf.callInitFarmer(gf.farmer1Identity);
    await gf.callInitFarmer(gf.farmer2Identity);
    await gf.callDeposit(gf.jewel1Amount, gf.farmer1Identity);
    await gf.callDeposit(gf.jewel2Amount, gf.farmer2Identity);

    //try staking
    await gf.stakeAndVerify(gf.farmer1Identity);
    await gf.stakeAndVerify(gf.farmer1Identity);
  });

  it('large max counts - works ok', async () => {
    //farm
    const maxCounts = {
      maxFarmers: 4294967295,
      maxJewels: 4294967295,
      maxRarityPoints: 4294967295,
    };
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed, maxCounts);

    //farmers
    await gf.callInitFarmer(gf.farmer1Identity);
    await gf.callInitFarmer(gf.farmer2Identity);
    await gf.callDeposit(gf.jewel1Amount, gf.farmer1Identity);
    await gf.callDeposit(gf.jewel2Amount, gf.farmer2Identity);

    //try staking
    await gf.stakeAndVerify(gf.farmer1Identity);
    await gf.stakeAndVerify(gf.farmer1Identity);
  });

  it('updates max counts', async () => {
    await gf.callInitFarm(defaultFarmConfig, RewardType.Fixed);

    let farm = await gf.fetchFarm();
    assert.equal(farm.maxCounts.maxFarmers, 0);
    assert.equal(farm.maxCounts.maxJewels, 0);
    assert.equal(farm.maxCounts.maxRarityPoints, 0);

    const maxCounts = {
      maxFarmers: 123,
      maxJewels: 123,
      maxRarityPoints: 123,
    };

    await gf.callUpdateFarm(undefined, undefined, maxCounts);

    farm = await gf.fetchFarm();
    assert.equal(farm.maxCounts.maxFarmers, 123);
    assert.equal(farm.maxCounts.maxJewels, 123);
    assert.equal(farm.maxCounts.maxRarityPoints, 123);
  });
});
