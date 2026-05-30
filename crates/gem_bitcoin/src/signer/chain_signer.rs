use primitives::{BitcoinChain, ChainSigner, SignerError, SignerInput};

use crate::signer::{
    planner::{SpendPlan, SpendRequest, UtxoPlanner},
    transaction::sign_plan,
    zcash::branch_id_from_metadata,
};

pub struct BitcoinChainSigner {
    chain: BitcoinChain,
    replace_by_fee: bool,
}

impl BitcoinChainSigner {
    pub fn new(chain: BitcoinChain) -> Self {
        Self::new_with_rbf(chain, false)
    }

    pub fn new_with_rbf(chain: BitcoinChain, replace_by_fee: bool) -> Self {
        Self { chain, replace_by_fee }
    }

    fn sign_request(&self, request: SpendRequest, private_key: &[u8], zcash_branch_id: Option<u32>) -> Result<String, SignerError> {
        let plan: SpendPlan = UtxoPlanner::plan(request)?;
        sign_plan(self.chain, &plan, private_key, zcash_branch_id)
    }

    fn zcash_branch_id(&self, input: &SignerInput) -> Result<Option<u32>, SignerError> {
        match self.chain {
            BitcoinChain::Zcash => Ok(Some(branch_id_from_metadata(&input.metadata)?)),
            BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => Ok(None),
        }
    }
}

impl ChainSigner for BitcoinChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_request(SpendRequest::transfer(self.chain, input, self.replace_by_fee)?, private_key, self.zcash_branch_id(input)?)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Ok(vec![self.sign_request(
            SpendRequest::swap(self.chain, input, self.replace_by_fee)?,
            private_key,
            self.zcash_branch_id(input)?,
        )?])
    }
}
