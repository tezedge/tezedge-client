# Tezedge Client

CLI client/wallet for Tezos. Currently in **POC** phase, use **only** with testnet.

## Create Transaction In DelphiNet Testnet

Store for keys isn't yet implemented so for now keys are hardcoded.
That means transaction can only be created with this key (`--from` address):
**tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU**

To create a transaction run these commands in sequence:

1. `cargo build`
1. `./target/debug/cli transfer -E https://testnet-tezos.giganode.io --from tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU --to tz1KsL1FCHy5qD5Q32XtS6aMh3eXqYbprpNi --amount 1 --fee 0.01`

## Run tests with

```bash
cargo test
```