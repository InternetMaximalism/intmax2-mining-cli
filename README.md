# mining-cli

A CLI tool for automatic mining of ITX tokens.

## Overview

mining-cli is a tool that allows users to mine ITX tokens, leveraging a simplified version of Intmax2. By utilizing a simplified version of Intmax2 that focuses on deposit and private withdrawal functions, users can contribute to enhancing Ethereum's privacy while earning rewards in ITX tokens.

Learn more about mining [here](https://hackmd.io/zNLtkMXXSCernbkTf1BTrQ).

## System Requirements

Before you begin, ensure your system meets the following minimum specifications:

- **Memory**: 8GB RAM or more

- **CPU**: 4 cores, 2 GHz or higher

- **Operating System**: Windows, macOS, or Linux

> **Note**: Some versions of Windows may encounter compatibility issues. For troubleshooting, refer to the [Windows guide](docs/windows_guide.md).

## Getting Started

1. Download the CLI

   - Go to the [Releases](https://github.com/InternetMaximalism/intmax2-mining-cli/releases) page.

   - Download the ZIP file for your operating system.

   - Extract the ZIP file content.

2. Choose a Network to Start Mining:

   - [Quick Start for Base Mainnet](docs/base_mainnet_quickstart.md) (Available since 19th Oct, 2024 00:00 UTC)

3. Additional Resources:

   - [How to Add ITX Token to Your Wallet](docs/add_token_to_wallet.md)

   - [Migration Guide (Ethereum Mainnet → Base)](docs/migrate.md)

   - [Developer Documentation](docs/developer_docs.md)

   - [Terms of Use](docs/terms_of_use.md)
     <br><br>

## How It Works

1. **Automated Deposits and Withdrawals**: The CLI automatically transfers funds from your deposit address into a simplified Intmax2 system. After a random delay, the funds are withdrawn to your withdrawal address. Through Zero-Knowledge Proofs , the relationship between your deposit and withdrawal addresses remains confidential.

2. **Reward Mechanism**: By participating in these private asset transfers, you contribute to enhancing Ethereum's privacy ecosystem. As a reward for your contribution, you earn ITX tokens. These tokens are distributed every Monday at 00:00 UTC to your withdrawal address. However, please note that rewards are subject to a two-week delay. For example, mining activities completed on a Sunday will not be eligible for claiming on the following Monday but on the Monday two weeks later.

<div align="center">
  <img src="assets/diagram.png" width="800" alt="Mining diagram">
</div>
   <br><br>

## About Pending Deposits

ETH enters a pending state immediately after deposit. The admin evaluates it according to AML criteria, and if there are no issues, it is deposited into the simplified intmax2. Deposits rejected by AML criteria are automatically refunded to the deposit address during mining. Pending deposits can be cancelled by running in exit mode.
<br><br>

## Status Updates

While mining, you’ll see status updates like the following. This indicates the state of the deposit account:

```

Deposits: 3 (success: 2 pending: 1 rejected: 0 cancelled: 0) Withdrawn: 2 Eligible: 0 (claimed: 0)

```

The status message components are:

- Deposits: Total number of deposits

- Success: Number of successful deposits

- Pending: Number of deposits awaiting AML analysis

- Rejected: Number of deposits rejected by AML analysis

- Cancelled: Number of cancelled deposits

- Withdrawn: Number of withdrawals

- Eligible: Number of deposits eligible for ITX rewards

- Claimed: Number of deposits for which rewards have been claimed
  <br><br>

## Important Notes⚠️

### AML Verification

Money deposited into the simplified version of Intmax2 undergoes AML (Anti-Money Laundering) verification. Deposits from suspicious addresses or those made through mixing services like Tornado Cash will be rejected.
You can recover rejected funds by launching the CLI exit mode.

### Reward Eligibility

This mining is privacy mining, and addresses that compromise the privacy gained through mining **will be ineligible for mining rewards**.
Specifically, if there are direct or indirect transfers between deposit addresses and withdrawal addresses, the deposit address used for that mining will not be eligible for rewards. You can check whether an deposit address is eligible for rewards in the "Qualified" column after selecting the mode.

Here are examples of actions that would make an address **ineligible** for rewards:

- From a wallet A, deposit 1.01 ETH into deposit address #0, and a total of 0.98 ETH is withdrawn to the withdrawal address before mining ends. Then, send 0.98 ETH back to wallet A.

- Deposit 1.01 ETH into deposit address #0, and a total of 0.98 ETH is withdrawn to the withdrawal address before mining ends. Then, deposit this amount into deposit address #1 and mine again.

- Deposit 1.1 ETH into deposit address #0. After mining is completed, 0.09 ETH remains in deposit address #0, which is then sent to the withdrawal address.

<div align="center">
  <img src="assets/diagram2.jpg" width="800" alt="Mining diagram">
</div>
<br><br>

## FAQs

Q: Can I lose my mining funds?<br>
A: Your funds are safe as long as you don't lose your withdrawal private key.

Q: Is this process self-custodial?<br>
A: Yes, but the contract is currently upgradable. The intmax team plans to relinquish this ability soon.

Q: What are the costs associated with mining?<br>
A: Gas fees are incurred for each deposit, withdrawal, and claim. The gas fee for withdrawal is deducted from the withdrawn ETH.

Q: What actions will disqualify me from receiving ITX token rewards?<br>
A: Avoid actions that link your deposit and withdrawal addresses. For example, if you directly or indirectly transfer funds from your withdrawal address to your deposit address, you will not be eligible for ITX rewards. Also, using the funds in the withdrawal address for the next mining directly is considered a linking action.

Q: How do I stop the CLI?<br>
A: To stop the CLI, simply press Ctrl+C in the terminal where it's running. This will safely terminate the process. If there is a balance in intmax2, you can withdraw it by running in the exit mode.

Q: An error occurred during execution. What should I do?<br>
A: Feel free to run it again. It's designed to be safe for re-execution.

_[Find more FAQs here](docs/faq.md)_
