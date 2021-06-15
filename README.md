# Tezedge Client

CLI client/wallet for Tezos.


## Getting Started

1. [Install Prerequisites](#install-prerequisites)
1. [Build the package](#build-the-package)
1. Choose desired [node endpoint](#node-endpoint).
1. Proceed with using tezedge-client.

**Important**: Please make sure to read the [potential issues section](#potential-issues)!

## Install Prerequisites

1. Install Git (client)

    ```bash
    sudo apt install git
    ```

1. Install Rust (We recommend installing Rust through rustup.)

    ```bash
    sudo apt install curl
    # Run the following in your terminal, then follow the onscreen instructions.
    curl https://sh.rustup.rs -sSf | sh
    ```

1. Install Rust toolchain

    ```bash
    rustup toolchain install 1.52.1
    ```

1. Install **required OS libs**
    - Clang for linking:

      ```bash
      sudo apt install clang
      ```
    - Sodiumoxide package:

      ```bash
      sudo apt install pkg-config libsodium-dev
      ```
    - Other requirements:

      ```bash
      sudo apt install libhidapi-dev libudev-dev
      ```

## Build The Package

1. **Download Tezedge Client source code**
    ```bash
    git clone https://github.com/tezedge/tezedge-client
    cd tezedge-client
    ```
1. **Set desired Rust toolchain version for tezedge-client**

    ```bash
    rustup override set 1.52.1
    ```
1. **Build**

    In order to build the package, you first need to set an environment variable
    (otherwise it won't compile):
    ```bash
    export SODIUM_USE_PKG_CONFIG=1
    ```

    Then you can execute a build command:
    ```bash
    cargo build --release
    ```

    **Or** you can instead set it each time when calling `cargo build`:
    ```bash
    SODIUM_USE_PKG_CONFIG=1 cargo build --release
    ```

<br>

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

- **Trezor:**

  ![](/docs/transfer_trezor.gif?raw=true)

  ```bash
  tezedge-client transfer -E https://rpctest.tzbeta.net --trezor --from "m/44'/1729'/0'/0'" --to tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --amount 0.5 --fee 0.01
  ```

- **Ledger:**

  ![](/docs/transfer_ledger.gif?raw=true)

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
  tezedge-client delegate -E https://rpctest.tzbeta.net --trezor --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --fee 0.01
  ```

- **Ledger:**
  ```bash
  tezedge-client delegate -E https://rpctest.tzbeta.net --ledger --key-path "m/44'/1729'/0'/0'" --from KT1Nm9tCSMA6WS1LHerH5PTVRDzbTLLyM5xp --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr --fee 0.01
  ```

## Unsafe Transfer + Delegate using local wallet

Transfer + Delegate funds using local wallet, by passing in public and
private keys to the cli as command line arguments.

### Warning!
This should only be used for **testing purposes!** This command requires
keys to be passed as command line arguments which is very unsafe.

---

- **Transfer**:
  ```bash
  tezedge-client unsafe-transfer-local \
      -E https://rpctest.tzbeta.net \
      --private-key edsk3p1JnT4LFXuxmcddNoJ7J5u12T7423mshwEikWmcLJnf2XvH7t \
      --public-key edpkvLzwxgqDf9qp5vGq5UvTHLRvz54PXae1U4UhWSTdjzAiKJbbJB \
      --from tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU \
      --to tz1e6W1pk9kkrjVTRWYZwtVFSjQQgYBmbhFp \
      --fee 0.1 \
      --amount 10

  ```
- **Delegate**:
  ```bash
  tezedge-client unsafe-delegate-local \
      -E https://rpctest.tzbeta.net \
      --private-key edsk3p1JnT4LFXuxmcddNoJ7J5u12T7423mshwEikWmcLJnf2XvH7t \
      --public-key edpkvLzwxgqDf9qp5vGq5UvTHLRvz54PXae1U4UhWSTdjzAiKJbbJB \
      --from tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU \
      --to tz1R55a2HQbXUAzWKJYE5bJp3UvvawwCm9Pr \
      --fee 0.1
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
