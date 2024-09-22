# v1-mining-cli

<div style="background-color: #FFFDE7; border: 1px solid #FBC02D; border-radius: 4px; padding: 16px; margin-bottom: 20px;">
  <p style="color: #F57F17; font-weight: bold; margin: 0;">
    ⚠️ Note: This pre-release CLI tool currently operates on Sepolia testnet with ~5-minute transaction intervals, not the Ethereum mainnet described in this documentation; expect significant changes before production release.
  </p>
</div>

A CLI tool for automatic mining of ITX tokens.

## Overview

v1-mining-cli is a tool that allows users to mine ITX tokens by participating in a simplified version of intmax2. By utilizing a simplified version of intmax2 that focuses on deposit and private withdrawal functions, users can contribute to enhancing Ethereum's privacy while earning rewards in ITX tokens.

### Key Features

- Automated mining process (deposit and withdrawal)
- Weekly ITX token rewards

## System Requirements

### Minimum Requirements

- Memory: 8GB or more
- CPU: 4 cores or more, with a clock speed of 2 GHz or higher

## Installation

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
cd intmax2-mining-cli/v1-cli
cargo install --path .
```

3. Run the CLI:

```bash
cd intmax2-mining-cli/v1-cli
mining-cli
```

### For Mac

1. Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Follow steps 2 and 3 from the Ubuntu instructions.

## Updating the CLI

To update the v1-mining-cli to the latest version:

1. Navigate to the repository directory and pull the latest changes:

```bash
cd path/to/intmax2-mining-cli
git pull origin main
```

2. Rebuild and reinstall the CLI:

```bash
cd v1-cli
cargo install --path .
```

3. Restart the mining-cli if it's currently running.

Note: Always check the release notes or changelog for any important updates or breaking changes before updating.

## Resetting the CLI State

To reset the CLI state, simply delete the `v1-cli/data` directory. This will clear all stored data.

Note: Be cautious, as this will erase all local data. Ensure you have backups of any important information.

## How It Works

<div align="center">
  <img src="assets/diagram.png" width="800" alt="Mining diagram">
</div>

1. **Setup**: You need three Ethereum addresses:

- **Deposit address**: Where you initially deposit ETH for mining
- **Withdrawal address**: Where mined ETH is withdrawn to
- **Claim address**: Where you receive ITX token rewards

Additionally, you need a mainnet RPC URL. We strongly recommend using Alchemy's RPC (the free plan is sufficient). This is because it has a high limit for retrieving event logs.

Note: Users must create these new addresses themselves and input them into the CLI.

- Start the CLI and follow the instructions to set up your addresses.
- Deposit 1 ETH, 10 ETH, or 100 ETH + gas fee to your deposit address following the instructions in the CLI.
- Deposit gas fee to your claim address following the instructions in the CLI.

2. **Mining Process**:

- The CLI automatically deposits smaller amounts (0.1 or 1 ETH) into intmax2. The deposit amount can be configured through the CLI
- After a few hours, it withdraws these amounts to your withdrawal address.
- There's a limit to deposits per address; create a new deposit address if you reach the limit.

3. **Rewards**:

- Receive ITX tokens weekly in your claim address (available every Monday)
- Ensure your claim address has enough ETH for gas fees

## Operating Modes

The CLI has four operating modes:

1. **Mining mode**: Automatically handles deposits, withdrawals. Stops when the deposit limit is reached.
2. **Claim mode**: Only claims ITX tokens. Stops when there are no more ITX tokens to claim.
3. **Exit mode**: Only performs withdrawals and cancels pending deposits. No new deposits are made.
4. **Wait for Claim**: Claims currently available ITX tokens, and waits for the next claim period to claim ITX tokens.

Note: If you switch to the exit mode immediately after depositing, you may be refunded to the deposit address.

## Important Notes

- **Privacy is crucial**: Avoid actions that link your deposit and withdrawal addresses. If you link your deposit and withdrawal addresses, you will not be eligible for ITX rewards.
- **Do not** directly transfer funds between your old withdrawal and new deposit addresses

## FAQs

Q: Can I lose my mining funds?
A: Your funds are safe as long as you don't lose your deposit private key.

Q: Is this process self-custodial?
A: Yes, but the contract is currently upgradable. The intmax team plans to relinquish this ability soon.

Q: How much can I earn?
A: Earnings vary based on your contribution and overall network activity.

Q: How often should I update the CLI?
A: It's recommended to check for updates regularly, at least once a week, to ensure you have the latest features and security improvements.

Q: How do I stop the CLI?
A: To stop the CLI, simply press Ctrl+C in the terminal where it's running. This will safely terminate the process. If there is a balance in intmax2, you can withdraw it by running in Shutdown mode.

Q: What happens if I deposit more than the initial deposit amount?
A: It's not a problem if you deposit more than the initial deposit amount.
