# Tezedge Client

CLI client/wallet for Tezos.


## Getting Started

1. [Build the package](#build-the-package)
1. Choose desired [node endpoint](#node-endpoint).
1. Proceed with using tezedge-client.

**Important**: Please make sure to read the [potential issues section](#potential-issues)!

## Build The Package

You can build the package simply by running: `cargo build --release`.

Cli binary can be located at: `./target/release/tezedge-client`.

## Node Endpoint

In case of commands which interact with the node, `--endpoint` argument
is required. Endpoint is an url pointing to tezos node’s rpc.

Sample node endpoints:

**Mainnet**
  - https://rpc.tzbeta.net
  - https://api.tez.ie/rpc/mainnet
  - https://mainnet.smartpy.io
  - https://mainnet-tezos.giganode.io

**Testnet** (first 2 most stable)
  - https://rpctest.tzbeta.net
  - https://api.tez.ie/rpc/edonet
  - https://edonet.smartpy.io
  - https://testnet-tezos.giganode.io

## Get Address From Hardware Wallet

To get address (public key hash) for a given [key derivation path](https://learnmeabitcoin.com/technical/derivation-paths) run this command:

- **Trezor:**

  ![](/docs/get_address_trezor.gif?raw=true)

  ```bash
  tezedge-client address get --trezor --path "m/44'/1729'/0'/0'"
  ```

- **Ledger:**

  ![](/docs/get_address_ledger.gif?raw=true)

  ```bash
  tezedge-client address get --ledger --path "m/44'/1729'/0'/0'"
  ```

## Create a Transaction

<p align="center"><img src="/docs/create_transaction.gif?raw=true"/></p>

- **Trezor:**
  ```bash
  tezedge-client transfer -E https://rpctest.tzbeta.net --trezor --from "m/44'/1729'/0'/0'" --to tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --amount 0.5 --fee 0.01
  ```

- **Ledger:**
  ```bash
  tezedge-client transfer -E https://rpctest.tzbeta.net --ledger --from "m/44'/1729'/0'/0'" --to tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --amount 0.5 --fee 0.01
  ```

## Crate a Delegation

- **Trezor:**
  ```bash
  tezedge-client delegate -E https://rpctest.tzbeta.net --trezor --from "m/44'/1729'/0'/0'" --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --fee 0.01
  ```

- **Ledger:**
  ```bash
  tezedge-client delegate -E https://rpctest.tzbeta.net --ledger --from "m/44'/1729'/0'/0'" --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --fee 0.01
  ```

## Scriptless(KT1) Account

Before [005_babylon](https://tezos.gitlab.io/protocols/005_babylon.html)
protocol update, it was possible to transfer/delegate funds from **KT1**
scriptless accounts. After the update it's no longer possible though.
To not break pre-babylon scriptless contracts, they were replaced with
manager.tz, so transfer and delegation are still possible with such accounts.
**[See more details here](https://tezos.gitlab.io/protocols/005_babylon.html#replace-kt1-accounts-with-manager-tz-script).**

In order to transfer/delegate from such accounts using hardware wallets,
you will need to pass **key derivation path** as `--key-path` and **KT1 address**,
from which we are transferring, as `--from`.

Like this: `--key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp`


#### Transfer from KT1 account

- **Trezor:**
  ```bash
  tezedge-client transfer -E https://rpctest.tzbeta.net --trezor --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --amount 0.5 --fee 0.01
  ```

- **Ledger:**
  ```bash
  tezedge-client transfer -E https://rpctest.tzbeta.net --ledger --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --amount 0.5 --fee 0.01
  ```


#### Delegate from KT1 account

- **Trezor:**
  ```bash
  tezedge-client delegate -E https://rpctest.tzbeta.net --trezor --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --amount 0.5 --fee 0.01
  ```

- **Ledger:**
  ```bash
  tezedge-client delegate -E https://rpctest.tzbeta.net --ledger --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --amount 0.5 --fee 0.01
  ```

## Forging

Forging is the process of encoding data to Tezos native binary representation.

Operations are forged locally, hence rpc node doesn't need to be
trusted to ensure security.

## Gas and Fee Estimation

Gas is estimated by simulating operationusing [run_operation](https://tezos.gitlab.io/008/rpc.html#post-block-id-helpers-scripts-run-operation)
endpoint in the node. **100** gas is added for safety to the estimation.

For operations from [scriptless(KT1) accounts](#scriptlesskt1-account),
additional consant gas is required, hence added. [You can see that here](cli/src/common/estimate_gas_consumption.rs).

For estimating a fee, along with estimating gas, operation size in bytes
needs to be calculated. To do that, operation is first forged locally
and then it's size is measured, which gives us an accurate measurement
of the operation size in bytes.

[Here is formula of how fee is estimated](utils/src/estimate_operation_fee.rs).


## Fee Suggestions

When using the cli to create an operation, it's fee will be estimated
and you will be prompted whether or not you'd like to use the
**estimated minimal fee**, instead of the entered one.

- If you've specified `--fee` argument, default answer will be **No**.
  So by default, it will leave current fee as is.
- If you didn't, default answer will be **Yes**.
  So by default, minimal estimated fee will be used.

You can also enter **custom** fee if you input `C`.

**Note**: Estimated minimal fee is only an estimation. Based on current
      network congestion, the fee might be enough or it might not.

## Potential issues

- If fee is too low for the operation, it might be accepted, but will
  never be included in the blockchain. In such case, if you attempt
  to create another operation in a short interval, you might receive
  the following **counter error**:
  ```
  injecting operation failed! Reason: Unknown! Http status: (500, Internal Server Error),
  message: [{"kind":"temporary","id":"failure","msg":"Error while applying operation ooiHfNSn38ZBC4byYx1wYNN4D5QVgm2bUivT2oWa7vtJf95nQz6:\nbranch refused (Error:\n                  Counter 303582 already used for contract tz1Yx6DUHcxDz4ye5gXePAC3p2K36wLKKCJa (expected 303583)\n)"}]
  ```
  If you wait for couple of minutes before repeating the operation,
  next time it should work.

  This issue is not unique to **tezos-client**, it's general to tezos system.
  [You can see more on this issue here](https://gitlab.com/tezos/tezos/-/issues/644).

- Cli might show that the operation was successful, but it's possible
  for that operation to not end up in the blockchain. To make sure that
  it did, please check the url that the cli outputs and check for
  block confirmations on the page.

- Retry logic for communicating with the node is not implemented.
  So if request sent to the node fails, command will exit with error.

  **Important**: If injecting (step **[3/4]**) an operation was successful and failure
             happens after that, even though you got the error, it's possible
             that operation was successful. So before repeating an operation,
             **always** make sure to wait a little and check your account
             in block explorer(tzstats.com), to see if operation went through.

- Currently doesn't work with multiple devices connected. If just 1 **Trezor**
  and/or just 1 **Ledger** is connected, it works. It doesn't work though if
  1+ **Trezor** and/or 1+ **Ledger** is connected.
