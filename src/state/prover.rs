use std::sync::OnceLock;

use mining_circuit_v1::{
    claim::{claim_processor::ClaimProcessor, claim_wrapper_processor::ClaimWrapperProcessor},
    withdrawal::simple_withdrawal_wrapper_processor::SimpleWithdrawalWrapperProcessor,
};
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig};

type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

pub struct Prover {
    withdrawal_wrapper_processor: OnceLock<SimpleWithdrawalWrapperProcessor>,
    claim_processor: OnceLock<ClaimProcessor<F, C, D>>,
    claim_wrapper_processor: OnceLock<ClaimWrapperProcessor>,
}

impl Prover {
    pub fn new() -> Self {
        Self {
            withdrawal_wrapper_processor: OnceLock::new(),
            claim_processor: OnceLock::new(),
            claim_wrapper_processor: OnceLock::new(),
        }
    }

    pub fn withdrawal_wrapper_processor(&self) -> &SimpleWithdrawalWrapperProcessor {
        self.withdrawal_wrapper_processor
            .get_or_init(|| SimpleWithdrawalWrapperProcessor::new())
    }

    pub fn claim_processor(&self) -> &ClaimProcessor<F, C, D> {
        self.claim_processor.get_or_init(|| ClaimProcessor::new())
    }

    pub fn claim_wrapper_processor(&self) -> &ClaimWrapperProcessor {
        self.claim_wrapper_processor
            .get_or_init(|| ClaimWrapperProcessor::new(&self.claim_processor().claim_circuit))
    }
}
