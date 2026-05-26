use primitives::{Asset, BitcoinChain, SignerError, SignerInput, SwapProvider, TransactionInputType, TransactionLoadMetadata, UTXO, decode_hex, swap::SwapQuoteDataType};

#[derive(Debug, Clone)]
pub(crate) struct SpendRequest {
    pub(crate) chain: BitcoinChain,
    pub(crate) sender_address: String,
    pub(crate) destination_address: String,
    pub(crate) amount: u64,
    pub(crate) is_max: bool,
    pub(crate) replace_by_fee: bool,
    pub(crate) fee_rate: u64,
    pub(crate) memo: Option<Vec<u8>>,
    pub(crate) utxos: Vec<UTXO>,
}

impl SpendRequest {
    pub(crate) fn transfer(chain: BitcoinChain, input: &SignerInput, replace_by_fee: bool) -> Result<Self, SignerError> {
        let asset = match &input.input_type {
            TransactionInputType::Transfer(asset) => asset,
            _ => return SignerError::invalid_input_err("unsupported Bitcoin transaction type"),
        };
        validate_native_chain_asset(chain, asset, "unsupported Bitcoin asset transfer")?;

        Ok(Self {
            chain,
            sender_address: input.sender_address.clone(),
            destination_address: input.destination_address.clone(),
            amount: input.value_as_u64()?,
            is_max: input.is_max_value,
            replace_by_fee,
            fee_rate: spend_fee_rate(chain, input)?,
            memo: input.get_memo().map(|memo| memo.as_bytes().to_vec()),
            utxos: metadata_utxos(chain, &input.metadata)?,
        })
    }

    pub(crate) fn swap(chain: BitcoinChain, input: &SignerInput, replace_by_fee: bool) -> Result<Self, SignerError> {
        let swap = input
            .input_type
            .get_swap_data()
            .map_err(|_| SignerError::invalid_input("unsupported Bitcoin transaction type"))?;
        validate_native_chain_asset(chain, input.input_type.get_asset(), "unsupported Bitcoin swap asset")?;
        let memo = match &swap.data.data_type {
            SwapQuoteDataType::Transfer => swap.data.memo.as_ref().map(|memo| memo.as_bytes().to_vec()),
            SwapQuoteDataType::Contract => Some(decode_hex(&swap.data.data)?),
        };
        let is_max = match (&swap.data.data_type, swap.quote.provider_data.provider, swap.quote.use_max_amount) {
            // Chainflip vault swaps require a third change output as the refund address.
            (SwapQuoteDataType::Contract, SwapProvider::Chainflip, _) => false,
            (SwapQuoteDataType::Contract | SwapQuoteDataType::Transfer, _, Some(use_max)) => use_max,
            (SwapQuoteDataType::Contract | SwapQuoteDataType::Transfer, _, None) => input.is_max_value,
        };

        Ok(Self {
            chain,
            sender_address: input.sender_address.clone(),
            destination_address: swap.data.to.clone(),
            amount: swap
                .data
                .value
                .parse::<u64>()
                .map_err(|_| SignerError::invalid_input(format!("invalid {} swap amount", chain.get_chain())))?,
            is_max,
            replace_by_fee,
            fee_rate: spend_fee_rate(chain, input)?,
            memo,
            utxos: metadata_utxos(chain, &input.metadata)?,
        })
    }
}

fn metadata_utxos(chain: BitcoinChain, metadata: &TransactionLoadMetadata) -> Result<Vec<UTXO>, SignerError> {
    match (chain, metadata) {
        (BitcoinChain::Zcash, TransactionLoadMetadata::Zcash { utxos, .. }) => Ok(utxos.clone()),
        (BitcoinChain::Zcash, _) => SignerError::invalid_input_err("missing Zcash transaction metadata"),
        (_, TransactionLoadMetadata::Bitcoin { utxos }) => Ok(utxos.clone()),
        _ => SignerError::invalid_input_err("missing Bitcoin transaction metadata"),
    }
}

fn spend_fee_rate(chain: BitcoinChain, input: &SignerInput) -> Result<u64, SignerError> {
    let minimum_fee_rate = u64::try_from(chain.minimum_byte_fee()).map_err(|_| SignerError::invalid_input(format!("invalid {} minimum fee", chain.get_chain())))?;
    Ok(input.fee.gas_price_u64()?.max(minimum_fee_rate))
}

fn validate_native_chain_asset(chain: BitcoinChain, asset: &Asset, message: &'static str) -> Result<(), SignerError> {
    (asset.id.chain == chain.get_chain() && asset.id.is_native())
        .then_some(())
        .ok_or_else(|| SignerError::invalid_input(message))
}
