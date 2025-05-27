use alloy::{
    primitives::{Address, B256},
    providers::Provider as _,
};
use dialoguer::{Confirm, Select};

use crate::{
    external_api::contracts::utils::NormalProvider,
    services::{
        balance_transfer::balance_transfer,
        utils::{is_address_used, pretty_format_u256},
    },
    state::key::Key,
};

pub async fn export_deposit_accounts(
    provider: &NormalProvider,
    withdrawal_private_key: B256,
) -> anyhow::Result<()> {
    let key = Key::new(withdrawal_private_key, 0);
    let balance = provider.get_balance(key.deposit_address).await?;
    println!();
    println!(
        "Deposit Address: {:?} ({} ETH)",
        key.deposit_address,
        pretty_format_u256(balance),
    );
    println!("Private Key: {:?}", key.deposit_private_key);

    let do_transfer = Confirm::new()
        .with_prompt("Do you want to make transfers from this account?")
        .default(true)
        .interact()?;
    if do_transfer {
        transfer_instruction(withdrawal_private_key).await?;
    }
    Ok(())
}

async fn transfer_instruction(withdrawal_private_key: B256) -> anyhow::Result<()> {
    let key = Key::new(withdrawal_private_key, 0);
    let to_address: Address = dialoguer::Input::<String>::new()
        .with_prompt("Enter the address to transfer to")
        .validate_with(|input: &String| {
            let result: Result<Address, _> = input.parse();
            match result {
                Ok(to_address) => {
                    if to_address == key.withdrawal_address {
                        return Err("Cannot transfer to the withdrawal address".to_string());
                    }
                    // The deposit address is encrypted, so it is theoretically impossible to check for duplicates.
                    Ok(())
                }
                Err(_) => Err("Invalid address".to_string()),
            }
        })
        .interact()?
        .parse()
        .unwrap(); // safe to unwrap because of the validation
    let is_ok = Confirm::new()
        .with_prompt(format!(
            "Are you sure to transfer all ETH from {:?} to {:?}",
            key.deposit_address, to_address,
        ))
        .report(false)
        .default(true)
        .interact()?;
    if !is_ok {
        return Ok(());
    } else {
        balance_transfer(key.deposit_private_key, to_address).await?;
    }
    Ok(())
}

pub async fn legacy_export_deposit_accounts(
    provider: &NormalProvider,
    withdrawal_private_key: B256,
) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            if key_number == 0 {
                println!("No deposit accounts found.");
                return Ok(());
            }
            break;
        }
        let balance = provider.get_balance(key.deposit_address).await?;
        println!();
        println!(
            "Deposit Address #{}: {:?} ({} ETH)",
            key_number,
            key.deposit_address,
            pretty_format_u256(balance),
        );
        println!("Private Key: {:?}", key.deposit_private_key);
        key_number += 1;
    }

    let do_transfer = Confirm::new()
        .with_prompt("Do you want to make transfers from these accounts?")
        .default(true)
        .interact()?;

    if do_transfer {
        legacy_transfer_instruction(withdrawal_private_key, key_number).await?;
    }
    Ok(())
}

async fn legacy_transfer_instruction(
    withdrawal_private_key: B256,
    up_to_key_number: u64,
) -> anyhow::Result<()> {
    let deposit_addresses = (0..up_to_key_number)
        .map(|i| Key::new(withdrawal_private_key, i).deposit_address)
        .collect::<Vec<Address>>();

    loop {
        let items = (0..up_to_key_number)
            .map(|i| {
                let key = Key::new(withdrawal_private_key, i);
                format!("#{} {:?}", i, key.deposit_address)
            })
            .collect::<Vec<String>>();
        let selected = Select::new()
            .with_prompt("Select a deposit account to transfer from")
            .items(&items)
            .default(0)
            .interact()? as u64;
        let key = Key::new(withdrawal_private_key, selected);
        let to_address: Address = dialoguer::Input::<String>::new()
            .with_prompt("Enter the address to transfer to")
            .validate_with(|input: &String| {
                let result: Result<Address, _> = input.parse();
                match result {
                    Ok(to_address) => {
                        if to_address == key.withdrawal_address {
                            return Err("Cannot transfer to the withdrawal address".to_string());
                        }
                        if deposit_addresses.contains(&to_address) {
                            return Err("Cannot transfer to a deposit address".to_string());
                        }
                        Ok(())
                    }
                    Err(_) => Err("Invalid address".to_string()),
                }
            })
            .interact()?
            .parse()
            .unwrap(); // safe to unwrap because of the validation
        let is_ok = Confirm::new()
            .with_prompt(format!(
                "Are you sure to transfer all ETH from #{} {:?} to {:?}",
                selected, key.deposit_address, to_address,
            ))
            .report(false)
            .default(true)
            .interact()?;
        if !is_ok {
            continue;
        } else {
            balance_transfer(key.deposit_private_key, to_address).await?;
        }
        let do_more = Confirm::new()
            .with_prompt("Do you want to make more transfers?")
            .default(false)
            .interact()?;
        if !do_more {
            break;
        }
    }
    Ok(())
}
