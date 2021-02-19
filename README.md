# Tezedge Client

CLI client/wallet for Tezos. Currently in **POC** phase, use **only** with testnet.

## Build the package

You can build the package by simply running: `cargo build`.

## Create Transaction In DelphiNet Testnet

Store for keys isn't yet implemented so for now keys are hardcoded.
That means transaction can only be created with this key (`--from` address):
**tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU**

To create a transaction run this:

```bash
./target/debug/cli transfer -E https://testnet-tezos.giganode.io --from tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --to tz1KsL1FCHy5qD5Q32XtS6aMh3eXqYbprpNi --amount 1 --fee 0.01
```

## Get Address from Trezor

To get address (public key hash) from Trezor using [key derivation path](https://learnmeabitcoin.com/technical/derivation-paths) run this command:

```bash
./target/debug/cli address get --trezor --path "m/44'/1729'/0'"
```

## Run tests with

```bash
cargo test
```