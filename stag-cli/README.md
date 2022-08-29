# Stag CLI

This crate implements CLI client for IBC solo machine which can be used to interface with other machines & replicated
ledgers which speak IBC.

## Installing

Please refer [../README.md](../README.md#installing) for installation instructions.

## Usage

1. Configuring signers

   Before doing any operations using `stag`, we first need to configure signers for different chains. `stag` uses `yaml`
   file for configuring signer. To obtain a sample `signer.yaml`, run:

   ```shell
   stag signer sample-config
   ```

   - Default value for `hd_path` is: `m/44'/118'/0'/0/0`.
   - Defailt value for `account_prefix` is: `cosmos`.
   - Default value for `algo` is: `secp256k1`.

1. Adding chains

   To run IBC operations on chain, `stag` needs some basic configuration for that chain. `stag` uses `yaml` file for
   adding chains. To obtains a sample `chain.yaml`, run:

   ```shell
   stag core sample-chain-config
   ```

   Change the `trusted_height` and `trusted_hash` to your blockchain's trusted height and trusted hash and run:

   ```shell
   stag core add-chain <path to chain.yaml>
   ```

1. Connecting to chain

   To establish an IBC connection with a chain, run:

   ```shell
   stag core connect <chain_id>
   ```

   For more options, run:

   ```shell
   stag core connect --help
   ```

1. Creating channels

   To create an IBC channel with a chain, run:

   ```shell
   stag core channel create <channel_type> <chain_id>
   ```

   - `channel_type` can be one of `transfer` or `ica`.

   For more options, run:

   ```shell
   stag core channel create --help
   ```

1. Minting and buring tokens

   To mint tokens on-chain using solo machine, run:

   ```shell
   stag transfer mint <chain_id> <amount> <denom>
   ```

   For more options, run:

   ```shell
   stag transfer mint --help
   ```

   To burn tokens on-chain using solo machine, run:

   ```shell
   stag transfer burn <chain_id> <amount> <denom>
   ```

   For more options, run:

   ```shell
   stag transfer burn --help
   ```

   To fetch the final on-chain denom of your solo machine token, run:

   ```shell
   stag query ibc-denom <chain_id> <denom>
   ```

1. Sending tokens from interchain account

   To send some tokens from your interchain account, run:

   ```shell
   stag ica bank send <chain_id> <to_address> <amount> <denom>
   ```

   For more options, run:

   ```shell
   stag ica bank send --help
   ```

   To fetch your interchain account address, run:

   ```shell
   stag query ica-address <chain_id>
   ```

   For more options, run:

   ```shell
   stag query ica-address --help
   ```

1. Delegate and un-delegate tokens from interchain account

   To delegate tokens from interchain account to a validator, run:

   ```shell
   stag ica staking delegate <chain_id> <validator_address> <amount> <denom>
   ```

   For more options, run:

   ```shell
   stag ica staking delegate --help
   ```

   To un-delegate tokens to interchain account from a validator, run:

   ```shell
   stag ica staking undelegate <chain_id> <validator_address> <amount> <denom>
   ```

   For more options, run:

   ```shell
   stag ica staking undelegate --help
   ```

1. Querying data from solo machine

   There are multiple commands to query data from solo machine. For more information, run:

   ```shell
   stag query --help
   ```
