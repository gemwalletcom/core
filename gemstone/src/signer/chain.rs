use crate::{GemstoneError, models::transaction::GemTransactionLoadInput};
use gem_aptos::AptosChainSigner;
use gem_hypercore::signer::HyperCoreSigner;
use gem_solana::signer::SolanaChainSigner;
use gem_sui::signer::SuiChainSigner;
use primitives::{Chain, ChainSigner, SignerError, TransactionLoadInput};

#[derive(uniffi::Object)]
pub struct GemChainSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

#[uniffi::export]
impl GemChainSigner {
    #[uniffi::constructor]
    pub fn new(chain: Chain) -> Self {
        let signer: Box<dyn ChainSigner> = match chain {
            Chain::Aptos => Box::new(AptosChainSigner),
            Chain::HyperCore => Box::new(HyperCoreSigner),
            Chain::Sui => Box::new(SuiChainSigner),
            Chain::Solana => Box::new(SolanaChainSigner),
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

    pub fn sign_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch_message(message, private_key, "message", |signer, msg, key| signer.sign_message(msg, key))
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
            SignerError::SigningError(_) => unsupported_error(self.chain, action),
            other => GemstoneError::from(other),
        })
    }

    fn dispatch_message<T, F>(&self, message: Vec<u8>, private_key: Vec<u8>, action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &[u8], &[u8]) -> Result<T, SignerError>,
    {
        method(self.signer.as_ref(), &message, &private_key).map_err(|err| match err {
            SignerError::SigningError(_) => unsupported_error(self.chain, action),
            other => GemstoneError::from(other),
        })
    }
}

fn unsupported_error(chain: Chain, action: &str) -> GemstoneError {
    SignerError::SigningError(format!("{action} not supported for chain {:?}", chain)).into()
}
