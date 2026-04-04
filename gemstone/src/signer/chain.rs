use crate::{GemstoneError, models::transaction::GemSignerInput};
use gem_algorand::AlgorandChainSigner;
use gem_aptos::AptosChainSigner;
use gem_cosmos::signer::CosmosChainSigner;
use gem_evm::signer::EvmChainSigner;
use gem_hypercore::signer::HyperCoreSigner;
use gem_near::NearChainSigner;
use gem_solana::signer::SolanaChainSigner;
use gem_stellar::StellarChainSigner;
use gem_sui::signer::SuiChainSigner;
use gem_tron::TronChainSigner;
use primitives::{Chain, ChainSigner, ChainType, EVMChain, SignerError, SignerInput};

#[derive(uniffi::Object)]
pub struct GemChainSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

#[uniffi::export]
impl GemChainSigner {
    #[uniffi::constructor]
    pub fn new(chain: Chain) -> Self {
        let signer: Box<dyn ChainSigner> = match chain.chain_type() {
            ChainType::Ethereum => Box::new(EvmChainSigner::new(EVMChain::from_chain(chain).unwrap())),
            ChainType::Aptos => Box::new(AptosChainSigner),
            ChainType::HyperCore => Box::new(HyperCoreSigner),
            ChainType::Sui => Box::new(SuiChainSigner),
            ChainType::Solana => Box::new(SolanaChainSigner),
            ChainType::Tron => Box::new(TronChainSigner),
            ChainType::Cosmos => Box::new(CosmosChainSigner),
            ChainType::Algorand => Box::new(AlgorandChainSigner),
            ChainType::Near => Box::new(NearChainSigner),
            ChainType::Stellar => Box::new(StellarChainSigner),
            _ => todo!("Signer not implemented for chain {:?}", chain),
        };

        Self { chain, signer }
    }

    pub fn sign_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "transfer", |signer, tx, key| signer.sign_transfer(tx, key))
    }

    pub fn sign_token_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token transfer", |signer, tx, key| signer.sign_token_transfer(tx, key))
    }

    pub fn sign_nft_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "nft transfer", |signer, tx, key| signer.sign_nft_transfer(tx, key))
    }

    pub fn sign_swap(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "swap", |signer, tx, key| signer.sign_swap(tx, key))
    }

    pub fn sign_token_approval(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token approval", |signer, tx, key| signer.sign_token_approval(tx, key))
    }

    pub fn sign_stake(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "stake", |signer, tx, key| signer.sign_stake(tx, key))
    }

    pub fn sign_account_action(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "account action", |signer, tx, key| signer.sign_account_action(tx, key))
    }

    pub fn sign_perpetual(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "perpetual", |signer, tx, key| signer.sign_perpetual(tx, key))
    }

    pub fn sign_withdrawal(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "withdrawal", |signer, tx, key| signer.sign_withdrawal(tx, key))
    }

    pub fn sign_data(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "data", |signer, tx, key| signer.sign_data(tx, key))
    }

    pub fn sign_earn(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "earn", |signer, tx, key| signer.sign_earn(tx, key))
    }

    pub fn sign_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch_message(message, private_key, "message", |signer, msg, key| signer.sign_message(msg, key))
    }
}

impl GemChainSigner {
    fn dispatch<T, F>(&self, input: GemSignerInput, private_key: Vec<u8>, action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<T, SignerError>,
    {
        let tx_input: SignerInput = input.into();
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
