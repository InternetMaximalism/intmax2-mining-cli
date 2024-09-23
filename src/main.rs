use std::{fs::File, path::PathBuf};

use clap::{arg, command, Parser};
use cli::{console::print_error, run};
use simplelog::{Config, LevelFilter, WriteLogger};
use state::mode::RunMode;
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
    mode: RunMode,
}

#[tokio::main]
async fn main() {
    // parse args
    let args = Args::parse();

    // load config
    utils::config::Settings::new().expect("Failed to load config");

    // setup logging
    let log_path = PathBuf::from(format!("data/logs/{}.log", chrono::Utc::now().to_rfc3339()));
    create_file_with_content(&log_path, &[]).expect("Failed to create log file");
    let log_file = File::create(log_path).unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    // run the CLI
    match run(args.mode).await {
        Ok(_) => {}
        Err(e) => {
            print_error(format!("{:#}", e));
        }
    }
}