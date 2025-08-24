# cpts

Easy-to-use CLI that enables you to commit buy or sell transactions on the Solana blockchain. Utilizes [Quicknodes](https://www.quicknode.com/) and [Jupiter](https://jup.ag/).

## Setup
You must have a file named `config.json` with the following key/values:
```json
{
    "wallet_address": "<YOUR_WALLET_PUBLIC_ADDRESS>",
    "wallet_private_key": "<YOUR_WALLET_PRIVATE_KEY>",
    "quicknode_rpc_url": "<YOUR_QUICKNODE_RPC_HTTPS_URL>"
}
```

## Usage
```
Usage: cpts.exe [OPTIONS] --mint <MINT>        

Options:
  -m, --mint <MINT>    Shit-coin mint (address)
  -u, --units <UNITS>  Number of units (amount)
  -s, --simulate       Dry-run
  -h, --help           Print help
  ```

## Notes
- The only fee that ocurrs when using this tool is the usual transaction processing cost in SOL and fiat value (0.000105 SOL (~$0.02 at this time)).
- Not specifying `<UNITS>` tells the program to sell all of that token.
