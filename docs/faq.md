# Frequently Asked Questions(FAQ)


###  1. Why can’t I find my ETH after depositing to the CLI address
You probably sent ETH in the wrong network, you can retreive ETH back and send to the right network
  
### 2. How to recover funds when sent to the wrong network

Export private key first and  import the wallet account in an external wallet(e.g, INTMAX Wallet or Metamask). You cannow transfer the funds and send using the corrrect network.

### 3. Error: Internal error: Failed to claim: Error sending transaction: Revert (bytes(0x)) ERROR: Internal error: Failed to claim: Error sending transaction: MiddlewareError...(JsonRpcError { code: -32003, message: "insufficient funds  for gas * price + value: have 13090885838070 want 2612337489

This error occurs when the withdrawal address has insufficient balance and runs out of gas during ZKP claim verification.

To resolve this issue, please deposit enough ETH to the withdrawal address to cover the gas fees. 


### 4. Error: Sync Deposit Tree From Event Failed to sync deposit tree: Network error: failed to get deposit leaf inserted event

Update CLI version to the latest version from CLI or download latest version from [release page](https://github.com/InternetMaximalism/intmax2-mining-cli/releases). 


### 5. What happens to the dust ETH left in my wallet after mining?
You can withdraw remaining ETH in wallet by using the `Exit` command or export the wallet using the `Export` command.

### 6. What to do when my CLI closes during claiming?

This could be caused by several factors. Open CLI and try claiming again. If issues persist, contact support team with your error message.

### 7. When I claim my tokens, Why can’t I find them in my wallet?

Note that after mining, rewards are made available after 2 weeks. Import the token contract address to your wallet to see your tokens. Get contract address [here](/docs/add_token_to_wallet.md).


### 8. Why do other people get more tokens than others even though they are mining the same way?

Token reward is determined by several factors including: Number of people mining, number of tokens claimable that day and mining phase(the amount of claimable tokens reduces as number of phase increases).

### 9. What do these mean? Processing claim for short and processing claim for long

Processing claim for short and long term is relative to the two claiming stages which are 1-2 weeks and 3 months after deposit.

### 10. I overwrote one of my config with ETH in it. Is there a way to recover the deposit address of the overwritten CLI deposit address?

You can retrieve that account by modifying the account, i.e,  changing the withdrawal address to the one that was linked to the deposit address with ETH in it.


### 11. What actions will disqualify me from receiving ITX token rewards? 
All identifiable withdraw/deposit combinations will be subject to reduction. All identifiable withdraw/deposit combinations will be subject to reduction, as will any minings that do not contribute to privacy, i.e., circulation type2. We have decided to write more about this in [hackmd](https://hackmd.io/zNLtkMXXSCernbkTf1BTrQ).

###  12. Can I continue using my previous withdrawal address in my wallet to receive deposits and claim tokens once I am done mining, or do I need a new one?

It is recommended to use a new address as your withdrawal address for every config. This is to prevent any form of linking. 

### 13. Why I should give the private key of my withdrawal address and not the public key? This is basic security - do not share the private key with anyone

Yes. And you are not giving a key to anybody. It's non-custodial. Importing a key to a node program is really normal while we admit that not so many people of current miners have experiences of that.

### 14. Is using exchange(CEX) and circulation from CEX to deposit address and from withdrawal address to CEX qualified to get reward?

As long as your activity can't be tracked on chain, you're fine. The idea is to push privacy. There should be no link  that connects the sender and receiver.

