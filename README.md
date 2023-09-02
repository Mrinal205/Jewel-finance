# Jewel Farm 💎
_by Jewelworks_

Jewel Farm is a collection of on-chain Solana programs for NFT ("jewel" 💎) staking.

It consists of:

- Jewel Bank 🏦 - responsible for storing NFTs, lets you configure which mints are/not allowed into the vaults
- Jewel Farm 🧑‍🌾 - responsible for issuing rewards, lets you configure fixed/variable rates, lock up periods, fees, rarities & more

Jewel Bank is used under the hood by Jewel Farm.

# Official deployment 🚀

Both programs are now officially deployed across all 3 networks (mainnet, devnet, testnet):
```
bank: bankHHdqMuaaST4qQk6mkzxGeKPHWmqdgor6Gs8r88m
farm: farmL4xeBFVXJqtfxCzU9b28QACM7E2W2ctT6epAjvE
```

You can interact with them using this [front-end](https://www.jewelfarm.gg/) (or build your own).

# Deploy your own version 🛠

- `git clone` the repo 
- Make sure you have `solana-cli` installed, keypair configured, and at least 10 sol on devnet beforehand
- Update path to your keypair in `Anchor.toml` that begins with `wallet =`
- Run `anchor build` to build the programs
- We need to update the program IDs:
    - Run `solana-keygen pubkey ./target/deploy/jewel_bank-keypair.json` - insert the new Bank prog ID in the following locations:
        - `./Anchor.toml`
        - `./programs/jewel_bank/src/lib.rs`
        - `./src/index.ts` (replace JEWEL_BANK_PROG_ID)
    - And `solana-keygen pubkey ./target/deploy/jewel_farm-keypair.json` - insert the new Farm prog ID in the following locations:
        - `./Anchor.toml`
        - `./programs/jewel_farm/src/lib.rs`
        - `./src/index.ts` (replace JEWEL_FARM_PROG_ID)
- Run `anchor build` to build one more time
- Run `anchor deploy --provider.cluster devnet` to deploy to devnet
- Now copy the IDLs into the apps:
    - `cp ./target/idl/jewel_bank.json ./app/jewel-bank/public`
    - `cp ./target/idl/jewel_bank.json ./app/jewel-farm/public`
    - `cp ./target/idl/jewel_farm.json ./app/jewel-farm/public`
- alternatively you can run the script I prepared `./scripts/cp_idl.sh`
- (!) IMPORTANT - run `yarn` inside the root of the repo
- finally start the apps!
    - eg cd into `app/jewel-bank` and run yarn && yarn serve
- don't forget to open Chrome's console with `CMD+SHIFT+I` to get feedback from the app when you click buttons. It currently doesn't have a notifications system

Note that deploying your own version will cost you ~20 SOL.

# Debug cryptic errors ⚠️

If you get a cryptic error back that looks something like this: 
```
Transaction failed 0x1798
``` 
The steps to take are as follows:
- translate the 0x number into decimal (eg using [this](https://www.rapidtables.com/convert/number/hex-to-decimal.html?x=0x66)) - eg 0x1798 becomes 6040
- if the number is 6XXX, this is a custom error from the app. Go to errors.rs found [here](https://github.com/jewelworks/jewel-farm/blob/main/lib/jewel_common/src/errors.rs) and find the error numbered 40 (the remainder of the decimal)
- any other number besides 6XXX means an anchor error - go [here](https://github.com/project-serum/anchor/blob/master/lang/src/error.rs) to decipher it

# Docs ✏️

Extensive documentation is available [here](https://docs.jewelworks.gg/).

The answer you're looking for is probably there. Pls don't DM with random questions.

# License 🧾

MIT
