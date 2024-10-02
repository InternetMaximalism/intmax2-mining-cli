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
        .join(format!(
            "{}.txt",
            chrono::Local::now().format("%Y-%m-%d-%H:%M:%S")
        ))
}

#[tokio::main]
async fn main() {
    let mode = Args::parse().command;
    let is_interactive = mode.is_none();

    // load the .env file if not in interactive mode
    if !is_interactive {
        dotenv().ok();
    }

    // test loading the setting
    get_project_root().expect("Failed to get project root: cannot find mining-cli-root");

    create_file_with_content(&get_log_file_path(), &[0]).expect("Failed to create log file");
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
