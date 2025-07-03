use console::{style, Term};
use dialoguer::Select;

use crate::state::mode::RunMode;

pub fn legacy_select_mode() -> anyhow::Result<RunMode> {
    let items = [
        format!(
            "{} {}",
            style("Claim:").bold(),
            style("claims available ITX tokens").dim()
        ),
        format!(
            "{} {}",
            style("Exit:").bold(),
            style("withdraws all balances currently and cancels pending deposits").dim()
        ),
        format!(
            "{} {}",
            style("Export:").bold(),
            style("export deposit private keys").dim()
        ),
        format!(
            "{} {}",
            style("Check Update:").bold(),
            style("check for updates of this CLI").dim()
        ),
    ];
    let term = Term::stdout();
    term.clear_screen()?;
    let mode = Select::new()
        .with_prompt("Select mode (press ctrl+c to abort)")
        .items(&items)
        .default(0)
        .interact()?;
    let mode = match mode {
        0 => RunMode::Claim,
        1 => RunMode::Exit,
        2 => RunMode::Export,
        3 => RunMode::CheckUpdate,
        _ => unreachable!(),
    };
    Ok(mode)
}
