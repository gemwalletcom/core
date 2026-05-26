use crate::{GemstoneError, models::transaction::GemSignerInput};
use gem_algorand::AlgorandChainSigner;
use gem_aptos::AptosChainSigner;
use gem_cardano::signer::CardanoChainSigner;
use gem_cosmos::signer::CosmosChainSigner;
use gem_evm::signer::EvmChainSigner;
use gem_hypercore::signer::HyperCoreSigner;
use gem_near::NearChainSigner;
use gem_polkadot::signer::PolkadotChainSigner;
use gem_solana::signer::SolanaChainSigner;
use gem_stellar::StellarChainSigner;
use gem_sui::signer::SuiChainSigner;
use gem_ton::signer::TonChainSigner;
use gem_tron::signer::TronChainSigner;
use gem_xrp::signer::XrpChainSigner;
use primitives::{Chain, ChainSigner, ChainType, EVMChain, SignerError, SignerInput};
use zeroize::Zeroizing;

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
            ChainType::Ton => Box::new(TonChainSigner),
            ChainType::Tron => Box::new(TronChainSigner),
            ChainType::Cosmos => Box::new(CosmosChainSigner),
            ChainType::Near => Box::new(NearChainSigner),
            ChainType::Algorand => Box::new(AlgorandChainSigner),
            ChainType::Stellar => Box::new(StellarChainSigner),
            ChainType::Xrp => Box::new(XrpChainSigner),
            ChainType::Polkadot => Box::new(PolkadotChainSigner),
            ChainType::Cardano => Box::new(CardanoChainSigner),
            _ => todo!("Signer not implemented for chain {:?}", chain),
        };

        Self { chain, signer }
    }

    pub fn sign_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "transfer", |signer, signer_input, key| signer.sign_transfer(signer_input, key))
    }

    pub fn sign_token_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token transfer", |signer, signer_input, key| {
            signer.sign_token_transfer(signer_input, key)
        })
    }

    pub fn sign_nft_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "nft transfer", |signer, signer_input, key| signer.sign_nft_transfer(signer_input, key))
    }

    pub fn sign_swap(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "swap", |signer, signer_input, key| signer.sign_swap(signer_input, key))
    }

    pub fn sign_token_approval(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token approval", |signer, signer_input, key| {
            signer.sign_token_approval(signer_input, key)
        })
    }

    pub fn sign_stake(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "stake", |signer, signer_input, key| signer.sign_stake(signer_input, key))
    }

    pub fn sign_account_action(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "account action", |signer, signer_input, key| {
            signer.sign_account_action(signer_input, key)
        })
    }

    pub fn sign_perpetual(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "perpetual", |signer, signer_input, key| signer.sign_perpetual(signer_input, key))
    }

    pub fn sign_withdrawal(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "withdrawal", |signer, signer_input, key| signer.sign_withdrawal(signer_input, key))
    }

    pub fn sign_data(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "data", |signer, signer_input, key| signer.sign_data(signer_input, key))
    }

    pub fn sign_earn(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "earn", |signer, signer_input, key| signer.sign_earn(signer_input, key))
    }

    pub fn sign_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let private_key = Zeroizing::new(private_key);
        self.dispatch_message(&message, private_key.as_slice(), "message", |signer, msg, key| signer.sign_message(msg, key))
    }
}

impl GemChainSigner {
    fn dispatch<T, F>(&self, input: GemSignerInput, private_key: Vec<u8>, action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<T, SignerError>,
    {
        let signer_input: SignerInput = input.into();
        let private_key = Zeroizing::new(private_key);

        method(self.signer.as_ref(), &signer_input, private_key.as_slice()).map_err(|err| map_signer_error(self.chain, action, err))
    }

    fn dispatch_message<T, F>(&self, message: &[u8], private_key: &[u8], action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &[u8], &[u8]) -> Result<T, SignerError>,
    {
        method(self.signer.as_ref(), message, private_key).map_err(|err| map_signer_error(self.chain, action, err))
    }
}

fn map_signer_error(chain: Chain, action: &str, error: SignerError) -> GemstoneError {
    match error {
        SignerError::SigningError(message) if message == format!("sign_{} not implemented", action.replace(' ', "_")) => unsupported_error(chain, action),
        error => GemstoneError::from(error),
    }
}

fn unsupported_error(chain: Chain, action: &str) -> GemstoneError {
    SignerError::SigningError(format!("{action} not supported for chain {:?}", chain)).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_signer_error() {
        assert_eq!(
            map_signer_error(Chain::Solana, "stake", SignerError::SigningError("sign_stake not implemented".to_string())).to_string(),
            "Signing error: stake not supported for chain solana"
        );
        assert_eq!(
            map_signer_error(
                Chain::Solana,
                "token transfer",
                SignerError::SigningError("sign_token_transfer not implemented".to_string())
            )
            .to_string(),
            "Signing error: token transfer not supported for chain solana"
        );
        assert_eq!(
            map_signer_error(Chain::Solana, "stake", SignerError::signing_error("sign: invalid private key")).to_string(),
            "Signing error: sign: invalid private key"
        );
    }
}
