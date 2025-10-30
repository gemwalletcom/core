use crate::{GemstoneError, models::transaction::GemTransactionLoadInput};
use gem_hypercore::signer::HyperCoreSigner;
use primitives::{Chain, ChainSigner, SignerError, TransactionLoadInput};

#[derive(uniffi::Object)]
pub struct GemChainSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

impl Default for GemChainSigner {
    fn default() -> Self {
        Self::new(Chain::HyperCore)
    }
}

#[uniffi::export]
impl GemChainSigner {
    #[uniffi::constructor]
    pub fn new(chain: Chain) -> Self {
        let signer = match chain {
            Chain::HyperCore => Box::new(HyperCoreSigner::new()) as Box<dyn ChainSigner>,
            _ => todo!("Signer not implemented for chain {:?}", chain),
        };

        Self { chain, signer }
    }

    pub fn sign_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "transfer", |signer, tx, key| signer.sign_transfer(tx, key))
    }

    pub fn sign_token_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token transfer", |signer, tx, key| signer.sign_token_transfer(tx, key))
    }

    pub fn sign_nft_transfer(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "nft transfer", |signer, tx, key| signer.sign_nft_transfer(tx, key))
    }

    pub fn sign_swap(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "swap", |signer, tx, key| signer.sign_swap(tx, key))
    }

    pub fn sign_token_approval(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token approval", |signer, tx, key| signer.sign_token_approval(tx, key))
    }

    pub fn sign_stake(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "stake", |signer, tx, key| signer.sign_stake(tx, key))
    }

    pub fn sign_account_action(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "account action", |signer, tx, key| signer.sign_account_action(tx, key))
    }

    pub fn sign_perpetual(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "perpetual", |signer, tx, key| signer.sign_perpetual(tx, key))
    }

    pub fn sign_withdrawal(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "withdrawal", |signer, tx, key| signer.sign_withdrawal(tx, key))
    }

    pub fn sign_data(&self, input: GemTransactionLoadInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "data", |signer, tx, key| signer.sign_data(tx, key))
    }
}

impl GemChainSigner {
    fn dispatch<T, F>(&self, input: GemTransactionLoadInput, private_key: Vec<u8>, action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &TransactionLoadInput, &[u8]) -> Result<T, SignerError>,
    {
        let tx_input: TransactionLoadInput = input.into();
        let key = private_key;

        method(self.signer.as_ref(), &tx_input, key.as_slice()).map_err(|err| match err {
            SignerError::UnsupportedOperation(_) => unsupported_error(self.chain, action),
            other => GemstoneError::from(other),
        })
    }
}

fn unsupported_error(chain: Chain, action: &str) -> GemstoneError {
    SignerError::UnsupportedOperation(format!("{action} not supported for chain {:?}", chain)).into()
}
