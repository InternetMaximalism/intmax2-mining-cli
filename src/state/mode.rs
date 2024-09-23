use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, ValueEnum)]
pub enum RunMode {
    Mining, // only mining
    Claim,  // only claim
    Exit,   // only withdraw or cancel pending deposits
}
