# `bitcoin_addresses`

Simple example project to demonstrate how to use [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) with the Threshold Signing API of the Internet Computer to generate different types of Bitcoin addresses. All addresses require just a signature from the Threshold Signing API to be spent.


## Supported address types

- P2PKH (Pay to Public Key Hash)
- P2SH (Pay to Script Hash)
- P2WPKH (Pay to Witness Public Key Hash)
- P2WSH (Pay to Witness Script Hash)
- P2TR (Pay to Taproot)

The first 4 address types use the Threshold ECDSA API. P2TR uses the Threshold Schnorr API which is currently in development, hence the [Schnorr API Developer Preview Canister](https://github.com/domwoe/schnorr_canister) is used. 

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```
You'll be asked to provide an initialization argument that defines which threshold master keys are used to derive the addresses.
- `Dfx` will use `dfx_test_key` which is available on the local replica.
- `Prod` will use `test_key1` available on ICP mainnet.
- `Test` will use `key1` available on ICP mainnet.

Once the job is completed, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

## Additional Resources

- [Bitcoin Addresses](https://en.bitcoin.it/wiki/Address).
- [ICP Bitcoin Integration Docs](https://internetcomputer.org/docs/current/developer-docs/multi-chain/bitcoin/overview).
