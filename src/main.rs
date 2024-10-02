use clap::{arg, command, Parser};
use cli::{console::print_error, run};
use dotenv::dotenv;
use external_api::github::fetch_config_file_from_github;
use simplelog::{Config, LevelFilter, WriteLogger};
use state::mode::RunMode;
use std::{fs::File, path::PathBuf};
use utils::file::{create_file_with_content, get_data_path};

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

    // load the .env file if not in interactive mode
    if !is_interactive {
        dotenv().ok();
    }
    let log_file_path = get_log_file_path().expect("Failed to get log file path");
    create_file_with_content(&log_file_path, &[]).expect("Failed to create log file");
    let log_file = File::create(log_file_path).unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    fetch_config_file_from_github()
        .await
        .expect("Failed to fetch config file from GitHub");

    // run the CLI
    match run(mode).await {
        Ok(_) => {}
        Err(e) => {
            print_error(format!("{}", e.to_string()));
        }
    }
}
