use clap::{arg, command, Parser};
use cli::{console::print_error, run};
use dotenv::dotenv;
use log::error;
use simplelog::{Config, LevelFilter, WriteLogger};
use state::mode::RunMode;
use std::{fs::File, path::PathBuf};
use utils::file::create_file_with_content;

pub mod cli;
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mode = Args::parse().command.unwrap_or(RunMode::Config);

    // load config
    utils::config::Settings::load().expect("Failed to load config");

    // setup logging
    let log_path = PathBuf::from(format!("data/logs/{}.log", chrono::Utc::now().to_rfc3339()));
    create_file_with_content(&log_path, &[]).expect("Failed to create log file");
    let log_file = File::create(log_path).unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    // run the CLI
    match run(mode).await {
        Ok(_) => {}
        Err(e) => {
            error!("{:#}", e);
            print_error(format!("{}", e.to_string()));
        }
    }
}
