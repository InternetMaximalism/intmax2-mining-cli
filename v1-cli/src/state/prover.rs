use mining_circuit::{
    claim::{claim_processor::ClaimProcessor, claim_wrapper_processor::ClaimWrapperProcessor},
    withdrawal::simple_withdrawal_wrapper_processor::SimpleWithdrawalWrapperProcessor,
};
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig};

type F = GoldilocksField;
const D: usize = 2;
type C = PoseidonGoldilocksConfig;

pub struct Prover {
    pub withdrawal_wrapper_processor: SimpleWithdrawalWrapperProcessor,
    pub claim_processor: ClaimProcessor<F, C, D>,
    pub claim_wrapper_processor: ClaimWrapperProcessor,
}

impl Prover {
    pub fn new() -> Self {
        let withdrawal_wrapper_processor = SimpleWithdrawalWrapperProcessor::new();
        let claim_processor = ClaimProcessor::new();
        let claim_wrapper_processor = ClaimWrapperProcessor::new(&claim_processor.claim_circuit);
        Self {
            withdrawal_wrapper_processor,
            claim_processor,
            claim_wrapper_processor,
        }
    }
}
