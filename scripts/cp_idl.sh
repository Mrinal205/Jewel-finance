# ------- copy IDLs into apps
# bank
cp ./target/idl/jewel_bank.json ./app/jewel-bank/public/
# farm
cp ./target/idl/jewel_bank.json ./app/jewel-farm/public/
cp ./target/idl/jewel_farm.json ./app/jewel-farm/public/

# ------- copy types into SDK
cp -r ./target/types ./src/

echo IDLs and Types copied!