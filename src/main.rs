use clap::{arg, command, Parser};
use cli::{console::print_error, run};
use dotenv::dotenv;
use log::error;
use simplelog::{Config, LevelFilter, WriteLogger};
use state::mode::RunMode;
use std::{fs::File, path::PathBuf};
use utils::file::{create_file_with_content, get_project_root, DATA_DIR};

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

fn get_log_file_path() -> PathBuf {
    get_project_root()
        .unwrap()
        .join(DATA_DIR)
        .join("logs")
        .join(format!("{}.log", chrono::Utc::now().to_rfc3339()))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mode = Args::parse().command.unwrap_or(RunMode::Interactive);

    // test loading the setting
    get_project_root().expect("Failed to get project root: cannot find mining-cli-root");
    utils::config::Settings::load().expect("Failed to load config");

    create_file_with_content(&get_log_file_path(), &[]).expect("Failed to create log file");
    let log_file = File::create(get_log_file_path()).unwrap();
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
