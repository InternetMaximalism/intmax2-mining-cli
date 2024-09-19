# v1-mining-cli

A CLI tool for automatic mining of INX tokens

## Overview

v1-mining-cli is a tool that allows users to mine ITX tokens by participating in a simplified version of intmax2. By utilizing a simplified version of intmax2 that focuses on deposit and private withdrawal functions, users can contribute to enhancing Ethereum's privacy while earning rewards in ITX tokens.

### Key Features

- Automated mining process (deposit and withdrawal)
- Weekly ITX token rewards

## Installation

The following instructions are for Ubuntu 24.04, but similar steps can be followed for other platforms.

### For Ubuntu 24.04

1. Install Rust and required packages:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt update && sudo apt install build-essential pkg-config libssl-dev
```

2. Clone the repository and install the CLI:

```bash
git clone https://github.com/internetMaximalism/intmax2-mining-cli.git
cd v1-cli
cargo install --path .
```

3. Run the CLI:

```bash
cd intmax2-mining-cli/v1-cli
mining-cli
```

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

1. **Setup**: You need three Ethereum addresses:

- **Deposit address**: Where you initially deposit ETH for mining
- **Withdrawal address**: Where mined ETH is withdrawn to
- **Claim address**: Where you receive ITX token rewards

2. **Mining Process**:

- Deposit 1 ETH, 10 ETH, or 100 ETH to your deposit address
- The CLI automatically deposits smaller amounts (0.1 or 1 ETH) into intmax2
- After a few hours, it withdraws these amounts to your withdrawal address

3. **Rewards**:

- Receive ITX tokens weekly in your claim address
- Ensure your claim address has enough ETH for gas fees

## Important Notes

- **Privacy is crucial**: Avoid actions that link your deposit and withdrawal addresses
- There's a limit to deposits per address; create a new deposit address if needed
- **Do not** directly transfer funds between your old withdrawal and new deposit addresses

## FAQs

Q: Can I lose my mining funds?
A: Your funds are safe as long as you don't lose your deposit private key.

Q: Is this process self-custodial?
A: Yes, but the contract is currently upgradable. The Intmax team plans to relinquish this ability soon.

Q: How much can I earn?
A: Earnings vary based on your contribution and overall network activity.

Q: How often should I update the CLI?
A: It's recommended to check for updates regularly, at least once a week, to ensure you have the latest features and security improvements.

Q: How do I stop the CLI?
A: To stop the CLI, simply press Ctrl+C in the terminal where it's running. This will safely terminate the process. If there is a balance in intmax2, you can withdraw it by running in Shutdown mode.
