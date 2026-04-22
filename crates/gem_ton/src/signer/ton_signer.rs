use primitives::SignerError;
use signer::Ed25519KeyPair;

use super::transaction::WalletV4R2;

pub(crate) struct TonSigner {
    key_pair: Ed25519KeyPair,
    wallet: WalletV4R2,
}

impl TonSigner {
    pub(crate) fn new(private_key: &[u8]) -> Result<Self, SignerError> {
        let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
        let wallet = WalletV4R2::new(key_pair.public_key_bytes)?;
        Ok(Self { key_pair, wallet })
    }

    pub(crate) fn wallet(&self) -> &WalletV4R2 {
        &self.wallet
    }

    #[cfg(test)]
    pub(crate) fn address(&self) -> &crate::address::Address {
        self.wallet.address()
    }

    pub(crate) fn sign(&self, digest: &[u8]) -> [u8; 64] {
        self.key_pair.sign(digest)
    }
}
