use clap::{arg, command, Parser};
use cli::{
    availability::check_availability, configure::select_network, console::print_error,
    press_enter_to_continue, run,
};
use dotenv::dotenv;
use simplelog::{Config, LevelFilter, WriteLogger};
use state::mode::RunMode;
use std::{env, fs::File, path::PathBuf};
use utils::{
    config::{create_config_files, Settings},
    file::{create_file_with_content, get_data_path},
};

pub mod cli;
pub mod constants;
pub mod external_api;
pub mod services;
pub mod state;
pub mod test;
pub mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The mode to run the program in
    #[arg(value_enum)]
    command: Option<RunMode>,
}

fn get_log_file_path() -> anyhow::Result<PathBuf> {
    Ok(get_data_path()?.join("logs").join(format!(
        "{}.log",
        chrono::Local::now().format("%Y-%m-%d-%H-%M-%S")
    )))
}

#[tokio::main]
async fn main() {
    let mode = Args::parse().command;
    let is_interactive = mode.is_none();

    match set_up(is_interactive).await {
        Ok(_) => {}
        Err(e) => {
            print_error(format!("Error during setup: {}", e));
            press_enter_to_continue();
            return;
        }
    }

    // run the CLI
    match run(mode).await {
        Ok(_) => {}
        Err(e) => {
            print_error(format!("{:?}", e));
            if is_interactive {
                // Because Windows closes the console window immediately, we need to wait for the user to see the error message
                press_enter_to_continue();
            }
        }
    }
}

async fn set_up(is_interactive: bool) -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    println!("Mining CLI {}", version);
    if is_interactive {
        // select network if in interactive mode
        let network = select_network()?;
        env::set_var("NETWORK", network.to_string());
    } else {
        // load the .env file if not in interactive mode
        dotenv().ok();
    }

    let log_file_path = get_log_file_path()?;
    create_file_with_content(&log_file_path, &[])?;
    let log_file = File::create(log_file_path)?;
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file)?;
    create_config_files()?;

    // check loading test
    let settings = Settings::load()?;
    log::info!(
        "Settings loaded: {}",
        serde_json::to_string_pretty(&settings)?
    );

    check_availability().await?;
    Ok(())
}
