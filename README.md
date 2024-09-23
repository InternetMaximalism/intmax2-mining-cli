# v1-mining-cli

<div style="background-color: #FFFDE7; border: 1px solid #FBC02D; border-radius: 4px; padding: 16px; margin-bottom: 20px;">
  <p style="color: #F57F17; font-weight: bold; margin: 0;">
  ⚠️ Note: This pre-release CLI tool currently operates on the Sepolia testnet. All references to "Mainnet" in the following README actually refer to the Sepolia testnet. This CLI tool only works on the Sepolia testnet, and there are the following differences compared to the future Mainnet release version:
  - Mining intervals occur every few minutes on Sepolia, whereas on Mainnet they will occur every few hours.
  - Claim intervals are daily with a one-day delay on Sepolia, whereas on Mainnet they will be weekly with a one-week delay.
  - On Sepolia, you receive test tokens instead of the ITX tokens that will be distributed on Mainnet.
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
cargo install --path .
```

3. Restart the mining-cli if it's currently running.

Note: Always check the release notes or changelog for any important updates or breaking changes before updating.

## Mining flow

<div align="center">
  <img src="assets/diagram.png" width="800" alt="Mining diagram">
</div>

1. **Preparation**: You need three types of Ethereum addresses:

- **Deposit address**: Where you initially deposit ETH for mining
- **Withdrawal address**: Where mined ETH is withdrawn to
- **Claim address**: Where you receive ITX token rewards

Additionally, you need a mainnet RPC URL. We strongly recommend using Alchemy's RPC (the free plan is sufficient). This is because it has a high limit for retrieving event logs. You can set these through environment variables. Please refer to the Operating Commands section below for more details.

2. **Mining Process**:

- The CLI automatically deposits smaller amounts (0.1 or 1 ETH) into intmax2. The deposit amount can be configured through environment variables.
- After a few hours, it withdraws these amounts to your withdrawal address.

3. **Rewards**:

- Receive ITX tokens weekly in your claim address (available every Monday. Rewards are delayed by one week. For example, mining done on a Sunday can be claimed not on the following Monday, but on the Monday 8 days later)
- Ensure your claim address has enough ETH for gas fees

## Operating Commands

The mining-cli has three main commands:

### 1. `mining-cli mining`

This command performs mining by repeatedly executing deposits and withdrawals.

Required environment variables:

- RPC_URL: Blockchain RPC URL. Alchemy's RPC is strongly recommended.
- MINING_UNIT: Amount of ETH per mining operation. Set to "0.1" or "1".
- MINING_TIMES: Number of mining operations (sets of deposit and withdrawal). Can be set to 10 or 100.
- DEPOSIT_PRIVATE_KEYS: Array of private keys for deposit accounts. Set in the format '["0xa...", "0xb..."]'. Each address must contain ETH equal to MINING_UNIT \* MINING_TIMES plus gas fees.
- WITHDRAWAL_ADDRESS: Address of the account for withdrawals. Balance can be 0 as gas fees are deducted from withdrawn ETH.

### 2. `mining-cli claim`

This command claims available ITX tokens.

Required environment variables:

- RPC_URL: Blockchain RPC URL. Alchemy's RPC is strongly recommended.
- DEPOSIT_PRIVATE_KEYS: Array of private keys used for deposits. Set in the format '["0xa...", "0xb..."]'. Balance can be 0.
- CLAIM_PRIVATE_KEY: Private key of the account used for claiming. Must contain enough ETH for gas fees.

### 3. `mining-cli exit`

This command withdraws all balances currently in the simplified intmax2 and cancels pending deposits.

Required environment variables:

- RPC_URL: Blockchain RPC URL. Alchemy's RPC is strongly recommended.
- DEPOSIT_PRIVATE_KEYS: Array of private keys used for deposits. Set in the format '["0xa...", "0xb..."]'.
- WITHDRAWAL_ADDRESS: Address of the account for withdrawals. Balance can be 0 as gas fees are deducted from withdrawn ETH.

## About Pending Deposits

ETH enters a pending state immediately after deposit. The admin evaluates it according to AML criteria, and if there are no issues, it is deposited into the simplified intmax2. Deposits rejected by AML criteria are automatically refunded to the deposit address during mining. Pending deposits can be cancelled by running in exit mode.

## Status

During mining, a status message like the following will be displayed. This indicates the state of the deposit account:

```
Deposits: 3 (success: 2 pending: 1 rejected: 0 cancelled: 0) Withdrawn: 2 Eligible: 0 (claimed: 0)
```

The status message components are:

- Success: Number of successful deposits
- Pending: Number of deposits awaiting AML analysis
- Rejected: Number of deposits rejected by AML analysis
- Cancelled: Number of cancelled deposits
- Withdrawn: Number of withdrawals
- Eligible: Number of deposits eligible for ITX rewards
- Claimed: Number of deposits for which rewards have been claimed

## Important Notes

- **Privacy is crucial**: Avoid actions that link your deposit and withdrawal addresses. If you link your deposit and withdrawal addresses, you will not be eligible for ITX rewards.
- **Do not** directly transfer funds between your old withdrawal and new deposit addresses

## FAQs

Q: Can I lose my mining funds?
A: Your funds are safe as long as you don't lose your deposit private key.

Q: Is this process self-custodial?
A: Yes, but the contract is currently upgradable. The intmax team plans to relinquish this ability soon.

Q: How much can I earn?
A: The amount of ITX tokens you can earn proportionally depends on the amount of ETH you deposit.

Q: How often should I update the CLI?
A: It's recommended to check for updates regularly, at least once a week, to ensure you have the latest features and security improvements.

Q: How do I stop the CLI?
A: To stop the CLI, simply press Ctrl+C in the terminal where it's running. This will safely terminate the process. If there is a balance in intmax2, you can withdraw it by running in the exit mode.

Q: What happens if I deposit more than the initial deposit amount?
A: It's not a problem if you deposit more than the initial deposit amount.
