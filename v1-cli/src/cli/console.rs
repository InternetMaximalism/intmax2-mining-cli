use console::{style, Term};
use ethers::{
    providers::Middleware as _,
    types::{Address, U256},
};
use log::{error, info, warn};

use crate::external_api::contracts::utils::get_client;

/// Print a colored status message to the console
/// This will overwrite the last line
pub fn print_status(message: &str) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();

    let colored_message = format!(
        "{} {}",
        style("STATUS:").green().bold(),
        style(message).blue()
    );

    term.write_line(&colored_message).unwrap();
    info!("{}", message);
}

pub fn print_warning(message: &str) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();

    let colored_message = format!(
        "{} {}",
        style("WARNING:").yellow().bold(),
        style(message).yellow()
    );
    term.write_line(&colored_message).unwrap();
    warn!("{}", message);
}

pub fn print_error(message: &str) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();

    let colored_message = format!("{} {}", style("ERROR:").red().bold(), style(message).red());
    term.write_line(&colored_message).unwrap();
    error!("{}", message);
}

pub async fn insuffient_balance_instruction(address: Address, name: &str) -> anyhow::Result<()> {
    let client = get_client().await?;
    let balance = client.get_balance(address, None).await?;
    print_warning(&format!(
        r"Insufficient balance of {} address {:?}. 
Current balance: {} ETH.
Waiting for your deposit...",
        name,
        address,
        pretty_format_u256(balance)
    ));
    loop {
        let new_balance = client.get_balance(address, None).await?;
        if new_balance > balance {
            print_status("Balance updated");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    Ok(())
}

pub fn pretty_format_u256(value: U256) -> String {
    let s = ethers::utils::format_units(value, "ether").unwrap();
    // remove trailing zeros
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pretty_format() {
        let value = ethers::utils::parse_ether("1.01000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1.01");

        let value = ethers::utils::parse_ether("1.00000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1");
    }
}
