use crate::{GemstoneError, models::transaction::GemTransactionLoadInput};
use gem_hypercore::signer::HypercoreSigner;
use primitives::{Chain, SignerError, TransactionLoadInput};

type ChainSignResult<T> = Result<T, SignerError>;

#[derive(uniffi::Object)]
pub struct GemChainSigner {
    chain: Chain,
    hypercore: HypercoreSigner,
}

#[uniffi::export]
impl GemChainSigner {
    #[uniffi::constructor]
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            hypercore: HypercoreSigner::new(),
        }
    }

    pub fn sign_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "transfer", HypercoreSigner::sign_transfer)
    }

    pub fn sign_token_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "token transfer", HypercoreSigner::sign_token_transfer)
    }

    pub fn sign_nft_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "nft transfer", HypercoreSigner::sign_nft_transfer)
    }

    pub fn sign_swap(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.route(input, private_key, "swap", HypercoreSigner::sign_swap)
    }

    pub fn sign_token_approval(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "token approval", HypercoreSigner::sign_token_approval)
    }

    pub fn sign_stake(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.route(input, private_key, "stake", HypercoreSigner::sign_stake)
    }

    pub fn sign_account_action(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "account action", HypercoreSigner::sign_account_action)
    }

    pub fn sign_perpetual(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.route(input, private_key, "perpetual", HypercoreSigner::sign_perpetual)
    }

    pub fn sign_withdrawal(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "withdrawal", HypercoreSigner::sign_withdrawal)
    }

    pub fn sign_data(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.route(input, private_key, "data", HypercoreSigner::sign_data)
    }
}

impl GemChainSigner {
    fn route<T, F>(&self, input: GemTransactionLoadInput, private_key: Vec<u8>, action: &'static str, f: F) -> Result<T, GemstoneError>
    where
        F: Fn(&HypercoreSigner, &TransactionLoadInput, &[u8]) -> ChainSignResult<T>,
    {
        let tx_input: TransactionLoadInput = input.into();
        let key = private_key;
        match self.chain {
            Chain::HyperCore => f(&self.hypercore, &tx_input, key.as_slice()).map_err(GemstoneError::from),
            _ => Err(unsupported_error(self.chain, action)),
        }
    }
}

fn unsupported_error(chain: Chain, action: &str) -> GemstoneError {
    SignerError::UnsupportedOperation(format!("{action} not supported for chain {:?}", chain)).into()
}
