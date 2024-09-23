#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Mining,       // only mining
    Claim,        // only claim
    Exit,         // only withdraw or cancel pending deposits
    WaitForClaim, // wait for claim
}
