use dialoguer::{Input, Select};
use ethers::{providers::Middleware as _, types::U256};
use tokio::time::sleep;

use crate::{
    cli::console::print_status,
    external_api::contracts::utils::get_client_with_rpc_url,
    services::contracts::pretty_format_u256,
    state::private_data::PrivateData,
    utils::config::{InitialDeposit, MiningAmount, Settings, UserSettings},
    utils::network::get_network,
};

use super::console::print_warning;

pub async fn user_settings(private_data: &PrivateData) -> anyhow::Result<()> {
    if !UserSettings::new().is_err() {
        // user settings already exists
        return Ok(());
    }

    let rpc_url = loop {
        let rpc_url: String = Input::new()
            .with_prompt(format!("RPC URL of {}", get_network()))
            .validate_with(|rpc_url: &String| {
                if rpc_url.starts_with("http") {
                    Ok(())
                } else {
                    Err("Invalid RPC URL")
                }
            })
            .interact()?;
        match check_rpc_url(&rpc_url).await {
            Ok(_) => break rpc_url,
            Err(e) => {
                print_warning(format!("{}", e));
                continue;
            }
        }
    };

    let mining_amount = {
        let items = vec!["0.1 ETH", "1.0 ETH"];
        let selection = Select::new()
            .with_prompt("Choose mining amount (single deposit amount)")
            .items(&items)
            .default(0)
            .interact()?;
        match selection {
            0 => MiningAmount::OneTenth,
            1 => MiningAmount::One,
            _ => unreachable!(),
        }
    };

    let initial_deposit = {
        let items = vec!["1 ETH", "10 ETH", "100 ETH"];
        let selection = Select::new()
            .with_prompt("Choose initial deposit")
            .items(&items)
            .default(0)
            .interact()?;
        match selection {
            0 => InitialDeposit::One,
            1 => InitialDeposit::Ten,
            2 => InitialDeposit::Hundred,
            _ => unreachable!(),
        }
    };

    let max_deposits = {
        let mining_amount = match mining_amount {
            MiningAmount::OneTenth => 0.1,
            MiningAmount::One => 1.0,
        };
        let initial_deposit = match initial_deposit {
            InitialDeposit::One => 1,
            InitialDeposit::Ten => 10,
            InitialDeposit::Hundred => 100,
        };
        (initial_deposit as f64 / mining_amount) as usize
    };

    initial_balance(private_data, &rpc_url, initial_deposit, max_deposits).await?;

    UserSettings {
        rpc_url,
        mining_amount,
        initial_deposit,
        max_deposits,
    }
    .save()?;
    Ok(())
}

async fn initial_balance(
    private_data: &PrivateData,
    rpc_url: &str,
    initial_deposit: InitialDeposit,
    num_deposits: usize,
) -> anyhow::Result<()> {
    let client = get_client_with_rpc_url(rpc_url).await?;
    let initial_deposit = match initial_deposit {
        InitialDeposit::One => ethers::utils::parse_ether("1").unwrap(),
        InitialDeposit::Ten => ethers::utils::parse_ether("10").unwrap(),
        InitialDeposit::Hundred => ethers::utils::parse_ether("100").unwrap(),
    };

    let settings = Settings::new()?;
    let single_deposit_gas_fee: U256 =
        U256::from_str_radix(&settings.blockchain.single_deposit_gas_fee, 10).unwrap();
    let single_claim_gas_fee =
        U256::from_str_radix(&settings.blockchain.single_claim_gas_fee, 10).unwrap();
    let min_deposit = initial_deposit + single_deposit_gas_fee * num_deposits;
    let min_claim = single_claim_gas_fee * num_deposits;

    let deposit_balance = client
        .get_balance(private_data.deposit_address, None)
        .await?;
    if deposit_balance < min_deposit {
        println!(
            "Deposit Address: {:?}  Balance: {} ETH",
            private_data.deposit_address,
            pretty_format_u256(deposit_balance)
        );
        println!(
            "Please deposit at least {} ETH to the above address",
            pretty_format_u256(min_deposit)
        );
        loop {
            let client = get_client_with_rpc_url(rpc_url).await?;
            let new_deposit_balance = client
                .get_balance(private_data.deposit_address, None)
                .await?;
            if new_deposit_balance >= min_deposit {
                print_status("Deposit completed");
                sleep(std::time::Duration::from_secs(5)).await;
                break;
            }
            sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    let claim_balance = client.get_balance(private_data.claim_address, None).await?;
    if claim_balance < min_claim {
        println!(
            "Claim Address: {:?} Balance: {} ETH",
            private_data.claim_address,
            pretty_format_u256(claim_balance)
        );
        println!(
            "Please deposit at least {} ETH as gas to the above address",
            pretty_format_u256(min_claim)
        );
        loop {
            let client = get_client_with_rpc_url(rpc_url).await?;
            let claim_balance = client.get_balance(private_data.claim_address, None).await?;
            if claim_balance >= min_claim {
                print_status("Deposit completed");
                sleep(std::time::Duration::from_secs(5)).await;
                break;
            }
            sleep(std::time::Duration::from_secs(5)).await;
        }
    }
    Ok(())
}

async fn check_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let client = ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)?;
    let chain_id = client.get_chainid().await?;
    let setting = Settings::new()?;
    if chain_id != setting.blockchain.chain_id.into() {
        return Err(anyhow::anyhow!(
            "RPC URL chain id {} does not match the expected chain id {}",
            chain_id.as_u64(),
            setting.blockchain.chain_id,
        ));
    }
    Ok(())
}
