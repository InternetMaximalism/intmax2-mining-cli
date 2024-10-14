use console::{style, Term};
use log::{error, info, warn};

pub fn initialize_console() {
    let term = Term::stdout();
    term.write_line("").unwrap();
}

/// Print a colored status message to the console
/// This will overwrite the last line
pub fn print_status<S: ToString>(message: S) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();
    let colored_message = format!(
        "{} {}",
        style("STATUS:").green().bold(),
        style(message.to_string()).blue()
    );
    term.write_line(&colored_message).unwrap();
    term.write_line("Press ctrl + c to stop the process")
        .unwrap();
    info!("{}", message.to_string());
}

// similar to print_status but not will be overwritten
pub fn print_log<S: ToString>(message: S) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();
    let colored_message = format!(
        "{} {}",
        style(format!("{}:", chrono::Local::now().format("%H:%M:%S"))).dim(),
        style(message.to_string()).blue()
    );
    term.write_line(&colored_message).unwrap();
    initialize_console();
    info!("{}", message.to_string());
}

pub fn print_warning<S: ToString>(message: S) {
    let term = Term::stdout();
    term.clear_last_lines(1).unwrap();
    let colored_message = format!(
        "{} {}",
        style("WARNING:").yellow().bold(),
        style(message.to_string()).yellow()
    );
    term.write_line(&colored_message).unwrap();
    term.write_line("Press ctrl + c to terminate the process")
        .unwrap();
    warn!("{}", message.to_string());
}



pub fn print_assets_status(assets_status: &crate::services::assets_status::AssetsStatus) {
    print_status(format!(
        "Deposits: {} (success: {} pending: {} rejected: {} cancelled: {}) Withdrawn: {}",
        assets_status.senders_deposits.len(),
        assets_status.contained_indices.len(),
        assets_status.pending_indices.len(),
        assets_status.rejected_indices.len(),
        assets_status.cancelled_indices.len(),
        assets_status.withdrawn_indices.len(),
    ));
}

pub fn print_error<S: ToString>(message: S) {
    let term = Term::stdout();
    let colored_message = format!(
        "{} {}",
        style("ERROR:").red().bold(),
        style(message.to_string()).red()
    );
    term.write_line(&colored_message).unwrap();
    term.write_line("").unwrap();
    error!("{}", message.to_string());
}
