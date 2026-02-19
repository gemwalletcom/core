use gem_hash::sha2::sha256;
use primitives::{ChainSigner, SignerError, TransactionLoadInput, TransferDataOutputType, hex::decode_hex};
use serde_json::Value;
use signer::{SignatureScheme, Signer};

enum PayloadFormat {
    V1,
    Legacy,
}

struct TronPayload {
    payload: Value,
    format: PayloadFormat,
    output_type: TransferDataOutputType,
}

impl TronPayload {
    fn parse(input: &TransactionLoadInput) -> Result<Self, SignerError> {
        let extra = input.get_data_extra().map_err(SignerError::invalid_input)?;
        let data = extra.data.as_ref().ok_or_else(|| SignerError::invalid_input("Missing transaction data"))?;
        let payload: Value = serde_json::from_slice(data)?;

        let transaction = payload
            .get("transaction")
            .ok_or_else(|| SignerError::invalid_input("Missing transaction in Tron payload"))?;

        let format = if transaction.get("raw_data_hex").is_some() {
            PayloadFormat::V1
        } else if transaction.get("transaction").and_then(|t| t.get("raw_data_hex")).is_some() {
            PayloadFormat::Legacy
        } else {
            return Err(SignerError::invalid_input("Missing raw_data_hex in Tron transaction payload"));
        };

        Ok(Self {
            payload,
            format,
            output_type: extra.output_type.clone(),
        })
    }

    fn transaction(&self) -> &Value {
        let transaction = &self.payload["transaction"];
        match self.format {
            PayloadFormat::V1 => transaction,
            PayloadFormat::Legacy => &transaction["transaction"],
        }
    }

    fn raw_data_hex(&self) -> &str {
        self.transaction()["raw_data_hex"].as_str().unwrap_or_default()
    }

    fn into_signed(self, signature_hex: &str) -> Result<String, SignerError> {
        let mut transaction = self.transaction().clone();
        transaction
            .as_object_mut()
            .ok_or_else(|| SignerError::invalid_input("Transaction is not an object"))?
            .insert("signature".to_string(), serde_json::json!([signature_hex]));
        serde_json::to_string(&transaction).map_err(Into::into)
    }
}

pub struct TronChainSigner;

impl ChainSigner for TronChainSigner {
    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let payload = TronPayload::parse(input)?;
        let raw_bytes = decode_hex(payload.raw_data_hex())?;
        let digest = sha256(&raw_bytes);
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, digest.to_vec(), private_key.to_vec()).map_err(|e| SignerError::signing_error(e.to_string()))?;
        let signature_hex = hex::encode(signature);

        match payload.output_type {
            TransferDataOutputType::Signature => Ok(signature_hex),
            TransferDataOutputType::EncodedTransaction => payload.into_signed(&signature_hex),
        }
    }
}
