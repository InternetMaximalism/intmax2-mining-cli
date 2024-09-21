use std::{fs::File, path::PathBuf};

use cli::{console::print_error, run};
use simplelog::{Config, LevelFilter, WriteLogger};
use utils::file::create_file_with_content;

pub mod cli;
pub mod config;
pub mod external_api;
pub mod private_data;
pub mod services;
pub mod state;
pub mod test;
pub mod utils;

#[tokio::main]
async fn main() {
    // load config
    config::Settings::new().expect("Failed to load config");

    // setup logging
    let log_path = PathBuf::from(format!("data/logs/{}.log", chrono::Utc::now().to_rfc3339()));
    create_file_with_content(&log_path, &[]).expect("Failed to create log file");
    let log_file = File::create(log_path).unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    // run the CLI
    match run().await {
        Ok(_) => {}
        Err(e) => {
            print_error(format!("{:#}", e));
        }
    }
}
