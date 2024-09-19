use std::fs::File;

use cli::{run, console::print_error};
use simplelog::{Config, LevelFilter, WriteLogger};

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
    let log_file = File::create("cli.log").unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    // run the CLI
    match run().await {
        Ok(_) => {}
        Err(e) => {
            print_error(&e.to_string());
        }
    }
}
