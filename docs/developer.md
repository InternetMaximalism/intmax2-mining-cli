# Guide for Developers

## Build from Source

### For Linux / Windows Subsystem for Linux (WSL)

1. Install required packages and Rust:

```bash
apt update && apt install -y git curl build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

These commands should be run with sudo if necessary, depending on the execution environment.

2. Clone the repository in arbitrary directory and install the CLI:

```bash
git clone https://github.com/internetMaximalism/intmax2-mining-cli.git
cd intmax2-mining-cli
cargo install --path .
```

3. Run the CLI:

```bash
cd intmax2-mining-cli
mining-cli --version
```

### For Mac

1. Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

1. Follow steps 2 and 3 from the Linux/WSL instructions.

## Updating the CLI

To update the v1-mining-cli to the latest version:

1. Navigate to the repository directory and pull the latest changes:

```bash
cd path/to/intmax2-mining-cli
git pull origin main
```

2. Rebuild and reinstall the CLI:

```bash
cargo install --path .
```

3. Restart the mining-cli if it's currently running.

## Operating Commands

The CLI can be operated interactively or run automatically by setting environment variables. Below is a list of environment variables required by the CLI.
Users utilizing the interactive mode do not need to set these environment variables.

### Environment Variables

| Name                               | Description                                                                                                                   | Example                                             | Default Value                       |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- | ----------------------------------- |
| `RPC_URL`                          | Blockchain RPC URL. Alchemy's RPC is strongly recommended. Required.                                                          | `https://eth-mainnet.alchemyapi.io/v2/YOUR-API-KEY` | None                                |
| `MAX_GAS_PRICE`                    | Maximum gas price in GWei allowed when executing transactions.                                                                | `30`                                                | `"30"` (mainnet), `"200"` (testnet) |
| `MINING_UNIT`                      | Amount of ETH per mining operation.                                                                                           | `"0.1"` or `"1"`                                    | `"0.1"`                             |
| `MINING_TIMES`                     | Number of mining operations (sets of deposit and withdrawal).                                                                 | `"10"` or `"100"`                                   | `"10"`                              |
| `WITHDRAWAL_PRIVATE_KEY`           | Private key of withdrawal address. Required when `ENCRYPT` is `false`.                                                        | `"0x789..."`                                        | None                                |
| `ENCRYPTED_WITHDRAWAL_PRIVATE_KEY` | Encrypted form of withdrawal private key. Required when `ENCRYPT` is `true`.                                                  | `"e356.."`                                          | None                                |
| `ENCRYPT`                          | Flag to specify whether to encrypt and store deposit private keys and withdrawal private key. Takes values "true" or "false". | `"true"` or `"false"`                               | `"true"`                            |

### Commands

1. `mining-cli mining`

   - Performs mining by repeatedly executing deposits and withdrawals.

2. `mining-cli claim`

   - Claims available ITX tokens.

3. `mining-cli exit`

   - Withdraws all balances currently in the simplified intmax2 and cancels pending deposits.

4. `mining-cli export`

   - Exports the all deposit private keys.
