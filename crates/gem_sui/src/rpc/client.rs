use std::{error::Error, str::FromStr, sync::Arc};

use gem_encoding::decode_base64;
use gem_encoding::protobuf::{MessageDecode, MessageEncode, decode_grpc_message, encode_grpc_message};
use gem_jsonrpc::grpc::GrpcTransport;
use num_bigint::BigInt;
use primitives::Chain;
use serde::de::DeserializeOwned;
use sui_types::Address;

use super::mapper::{map_checkpoint, map_executed_transaction, map_inspect_result, map_sui_effects};
use super::proto::{
    self as proto, BatchGetObjectsRequest, BatchGetObjectsResponse, BatchGetTransactionsRequest, BatchGetTransactionsResponse, ExecuteTransactionRequest,
    ExecuteTransactionResponse, FieldMask, GetBalanceRequest, GetBalanceResponse, GetCheckpointRequest, GetCheckpointResponse, GetCoinInfoRequest, GetCoinInfoResponse,
    GetEpochRequest, GetEpochResponse, GetFunctionRequest, GetFunctionResponse, GetObjectRequest, GetObjectResponse, GetServiceInfoRequest, GetServiceInfoResponse,
    GetTransactionRequest, GetTransactionResponse, ListBalancesRequest, ListBalancesResponse, ListOwnedObjectsRequest, ListOwnedObjectsResponse, SimulateTransactionRequest,
    SimulateTransactionResponse, Transaction as GrpcTransaction, TransactionChecks, UserSignature as GrpcUserSignature, WithMut,
};
use super::transport::default_transport;
use crate::models::transaction::{SuiBroadcastTransaction, SuiTransaction};
use crate::models::{Balance, Checkpoint, Coin, Digest, InspectResult, Object, OwnedCoins, SuiCoinMetadata, SuiObject, TransactionBlocks};
use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL};

const TRANSACTION_READ_MASK: &[&str] = &[
    "digest",
    "effects.gas_used",
    "effects.status",
    "effects.gas_object",
    "events.events.package_id",
    "events.events.event_type",
    "events.events.json",
    "balance_changes",
    "timestamp",
];

pub(super) const PATH_GET_EPOCH: &str = "/sui.rpc.v2.LedgerService/GetEpoch";
const PATH_GET_SERVICE_INFO: &str = "/sui.rpc.v2.LedgerService/GetServiceInfo";
const PATH_GET_OBJECT: &str = "/sui.rpc.v2.LedgerService/GetObject";
const PATH_GET_CHECKPOINT: &str = "/sui.rpc.v2.LedgerService/GetCheckpoint";
const PATH_GET_TRANSACTION: &str = "/sui.rpc.v2.LedgerService/GetTransaction";
const PATH_GET_FUNCTION: &str = "/sui.rpc.v2.MovePackageService/GetFunction";
pub(super) const PATH_LIST_OWNED_OBJECTS: &str = "/sui.rpc.v2.StateService/ListOwnedObjects";
const PATH_GET_BALANCE: &str = "/sui.rpc.v2.StateService/GetBalance";
const PATH_LIST_BALANCES: &str = "/sui.rpc.v2.StateService/ListBalances";
const PATH_GET_COIN_INFO: &str = "/sui.rpc.v2.StateService/GetCoinInfo";
const PATH_BATCH_GET_OBJECTS: &str = "/sui.rpc.v2.LedgerService/BatchGetObjects";
const PATH_BATCH_GET_TRANSACTIONS: &str = "/sui.rpc.v2.LedgerService/BatchGetTransactions";
pub(super) const PATH_SIMULATE_TRANSACTION: &str = "/sui.rpc.v2.TransactionExecutionService/SimulateTransaction";
pub(crate) const PATH_EXECUTE_TRANSACTION: &str = "/sui.rpc.v2.TransactionExecutionService/ExecuteTransaction";
const BATCH_GET_TRANSACTIONS_LIMIT: usize = 50;

#[derive(Clone, Debug)]
pub struct SuiClient {
    endpoint: String,
    transport: Option<Arc<dyn GrpcTransport>>,
}

impl SuiClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            transport: default_transport(),
        }
    }

    pub fn new_with_transport(endpoint: impl Into<String>, transport: Arc<dyn GrpcTransport>) -> Self {
        Self {
            endpoint: endpoint.into(),
            transport: Some(transport),
        }
    }

    pub(super) async fn grpc_unary<Req, Resp>(&self, path: &str, request: Req) -> Result<Resp, Box<dyn Error + Send + Sync>>
    where
        Req: MessageEncode,
        Resp: MessageDecode,
    {
        let transport = self.transport.as_ref().ok_or("missing Sui gRPC transport")?;
        let body = encode_grpc_message(&request);
        let response = transport.unary(&self.endpoint, path, body).await?;
        decode_grpc_message(&response).map_err(|error| format!("Sui gRPC decode failed for {path}: {error}").into())
    }

    pub async fn inspect_transaction_block(&self, sender: &str, tx_data: &[u8], _gas_price: Option<u64>) -> Result<InspectResult, Box<dyn Error + Send + Sync>> {
        let transaction = decode_inspect_transaction_bytes(sender, tx_data)?;
        let request = SimulateTransactionRequest::new(transaction).with(|request| {
            request.read_mask = Some(FieldMask::from_paths([
                "transaction.effects.gas_used",
                "transaction.effects.status",
                "command_outputs.return_values.value",
            ]));
            request.checks = Some(TransactionChecks::Disabled);
        });
        Ok(map_inspect_result(self.grpc_unary(PATH_SIMULATE_TRANSACTION, request).await?))
    }

    pub async fn get_balance(&self, address: String) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        self.get_balance_for_coin(&address, SUI_COIN_TYPE_FULL).await
    }

    pub async fn get_balance_for_coin(&self, address: &str, coin_type: &str) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        let request = GetBalanceRequest {
            owner: Some(address.to_string()),
            coin_type: Some(coin_type.to_string()),
        };
        let response: GetBalanceResponse = self.grpc_unary(PATH_GET_BALANCE, request).await?;
        let balance = response.balance.unwrap_or_default();
        Ok(Balance {
            coin_type: balance.coin_type.unwrap_or_else(|| coin_type.to_string()),
            total_balance: BigInt::from(balance.balance.unwrap_or_default()),
            address_balance: balance.address_balance.unwrap_or_default(),
        })
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        let mut request = ListBalancesRequest {
            owner: Some(address),
            page_size: Some(1000),
            ..Default::default()
        };
        let mut balances = Vec::new();

        loop {
            let response: ListBalancesResponse = self.grpc_unary(PATH_LIST_BALANCES, request.clone()).await?;
            let page = response
                .balances
                .into_iter()
                .map(|balance| {
                    Ok(Balance {
                        coin_type: balance.coin_type.ok_or("missing Sui balance coin type")?,
                        total_balance: BigInt::from(balance.balance.ok_or("missing Sui balance amount")?),
                        address_balance: balance.address_balance.unwrap_or_default(),
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?;
            balances.extend(page);
            if response.next_page_token.is_none() {
                break;
            }
            request.page_token = response.next_page_token;
        }

        Ok(balances)
    }

    pub async fn get_coin_metadata(&self, token_id: String) -> Result<SuiCoinMetadata, Box<dyn Error + Send + Sync>> {
        let request = GetCoinInfoRequest { coin_type: Some(token_id) };
        let response: GetCoinInfoResponse = self.grpc_unary(PATH_GET_COIN_INFO, request).await?;
        let metadata = response.metadata.ok_or("missing Sui coin metadata")?;
        Ok(SuiCoinMetadata {
            decimals: metadata.decimals.unwrap_or_default() as i32,
            name: metadata.name.unwrap_or_default(),
            symbol: metadata.symbol.unwrap_or_default(),
        })
    }

    pub async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        map_service_chain_identifier(self.service_info().await?)
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.service_info().await?.checkpoint_height.ok_or("missing Sui checkpoint height")?)
    }

    pub async fn get_gas_price(&self) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
        let request = GetEpochRequest {
            epoch: None,
            read_mask: Some(FieldMask::from_path_string("reference_gas_price")),
        };
        let response: GetEpochResponse = self.grpc_unary(PATH_GET_EPOCH, request).await?;
        let epoch = response.epoch.ok_or("missing Sui epoch")?;
        Ok(BigInt::from(epoch.reference_gas_price.ok_or("missing Sui reference gas price")?))
    }

    pub async fn get_coins(&self, address: &str, coin_type: &str) -> Result<OwnedCoins<Coin>, Box<dyn Error + Send + Sync>> {
        let (objects, balance) = futures::try_join!(self.list_coin_objects(address, coin_type), self.get_balance_for_coin(address, coin_type),)?;
        Ok(OwnedCoins::new(coin_type.to_string(), objects, balance.address_balance))
    }

    async fn list_coin_objects(&self, address: &str, coin_type: &str) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
        let mut request = ListOwnedObjectsRequest {
            owner: Some(address.to_string()),
            page_size: Some(1000),
            object_type: Some(format!("0x2::coin::Coin<{}>", full_sui_coin_type(coin_type))),
            read_mask: Some(FieldMask::from_paths(["object_id", "version", "digest", "balance", "object_type"])),
            ..Default::default()
        };
        let mut coins = Vec::new();

        loop {
            let response: ListOwnedObjectsResponse = self.grpc_unary(PATH_LIST_OWNED_OBJECTS, request.clone()).await?;
            let page = response
                .objects
                .into_iter()
                .map(|object| {
                    let coin_type = object
                        .object_type
                        .ok_or("missing Sui coin object type")?
                        .trim_start_matches("0x2::coin::Coin<")
                        .trim_end_matches('>')
                        .to_string();
                    let object_id_str = object.object_id.ok_or("missing Sui coin object id")?;
                    let digest_str = object.digest.ok_or("missing Sui coin digest")?;
                    Ok(Coin {
                        coin_type,
                        balance: object.balance.ok_or("missing Sui coin balance")?,
                        object: Object {
                            object_id: Address::from_str(&object_id_str)?,
                            digest: sui_types::Digest::from_str(&digest_str)?,
                            version: object.version.ok_or("missing Sui coin version")?,
                        },
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?;
            coins.extend(page);
            if response.next_page_token.is_none() {
                break;
            }
            request.page_token = response.next_page_token;
        }

        Ok(coins)
    }

    pub async fn get_object(&self, object_id: String) -> Result<SuiObject, Box<dyn Error + Send + Sync>> {
        let object_id = Address::from_str(&object_id)?;
        let request = GetObjectRequest::new(&object_id).with(|request| {
            request.read_mask = Some(FieldMask::from_paths(["object_id", "version", "digest"]));
        });
        let response: GetObjectResponse = self.grpc_unary(PATH_GET_OBJECT, request).await?;
        let object = response.object.ok_or("missing Sui object")?;
        Ok(SuiObject {
            object_id: object.object_id.ok_or("missing Sui object id")?,
            digest: object.digest.ok_or("missing Sui object digest")?,
            version: object.version.ok_or("missing Sui object version")?.to_string(),
        })
    }

    pub async fn get_object_json<T>(&self, object_id: String) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned,
    {
        let object_id = Address::from_str(&object_id)?;
        let request = GetObjectRequest::new(&object_id).with(|request| {
            request.read_mask = Some(FieldMask::from_paths(["json"]));
        });
        let response: GetObjectResponse = self.grpc_unary(PATH_GET_OBJECT, request).await?;
        let json = response.object.and_then(|object| object.json).ok_or("missing Sui object JSON")?;
        Ok(serde_json::from_value(json)?)
    }

    pub(crate) async fn get_function(&self, package_id: &str, module: &str, function: &str) -> Result<proto::FunctionDescriptor, Box<dyn Error + Send + Sync>> {
        let package_id = Address::from_str(package_id)?;
        let request = GetFunctionRequest::new(&package_id, module, function);
        let response: GetFunctionResponse = self.grpc_unary(PATH_GET_FUNCTION, request).await?;
        Ok(response.function.ok_or("missing Sui function descriptor")?)
    }

    pub async fn dry_run(&self, tx_data: String) -> Result<SuiTransaction, Box<dyn Error + Send + Sync>> {
        let transaction = decode_transaction_base64(&tx_data)?;
        let request = SimulateTransactionRequest::new(transaction).with(|request| {
            request.read_mask = Some(FieldMask::from_paths(["transaction.effects.gas_used", "transaction.effects.status"]));
            request.checks = Some(TransactionChecks::Enabled);
        });
        let response: SimulateTransactionResponse = self.grpc_unary(PATH_SIMULATE_TRANSACTION, request).await?;
        let executed = response.transaction.ok_or("missing simulated transaction")?;
        Ok(SuiTransaction {
            effects: map_sui_effects(executed.effects.as_ref()),
        })
    }

    pub async fn get_transaction(&self, transaction_id: String) -> Result<Digest, Box<dyn Error + Send + Sync>> {
        let request = GetTransactionRequest {
            digest: Some(transaction_id),
            read_mask: Some(FieldMask::from_paths(TRANSACTION_READ_MASK.iter().copied())),
        };
        let response: GetTransactionResponse = self.grpc_unary(PATH_GET_TRANSACTION, request).await?;
        map_executed_transaction(response.transaction.ok_or("missing Sui transaction")?)
    }

    pub async fn get_transactions_by_block(&self, checkpoint: u64) -> Result<Checkpoint, Box<dyn Error + Send + Sync>> {
        let request = GetCheckpointRequest::by_sequence_number(checkpoint).with(|request| {
            request.read_mask = Some(FieldMask::from_paths(["sequence_number", "digest", "summary", "contents.transactions.transaction"]));
        });
        let response: GetCheckpointResponse = self.grpc_unary(PATH_GET_CHECKPOINT, request).await?;
        let checkpoint = response.checkpoint.ok_or("missing Sui checkpoint")?;
        map_checkpoint(checkpoint)
    }

    pub async fn get_checkpoint_transactions(&self, checkpoint: u64, limit: Option<usize>) -> Result<TransactionBlocks, Box<dyn Error + Send + Sync>> {
        let checkpoint = self.get_transactions_by_block(checkpoint).await?;
        let digests: Vec<_> = checkpoint.transactions.into_iter().take(limit.unwrap_or(usize::MAX)).collect();
        if digests.is_empty() {
            return Ok(TransactionBlocks { data: Vec::new() });
        }

        let mut data = Vec::new();
        for digests in digests.chunks(BATCH_GET_TRANSACTIONS_LIMIT) {
            data.extend(self.batch_get_transactions(digests.to_vec()).await?);
        }
        Ok(TransactionBlocks { data })
    }

    async fn batch_get_transactions(&self, digests: Vec<String>) -> Result<Vec<Digest>, Box<dyn Error + Send + Sync>> {
        let request = BatchGetTransactionsRequest {
            digests,
            read_mask: Some(FieldMask::from_paths(TRANSACTION_READ_MASK.iter().copied())),
        };
        let response: BatchGetTransactionsResponse = self.grpc_unary(PATH_BATCH_GET_TRANSACTIONS, request).await?;
        response
            .transactions
            .into_iter()
            .map(|result| match result {
                proto::GetTransactionResult::Transaction(transaction) => map_executed_transaction(*transaction),
                proto::GetTransactionResult::Error(status) => Err(format!("Sui transaction gRPC error {}: {}", status.code, status.message).into()),
                proto::GetTransactionResult::Unknown => Err("missing Sui transaction result".into()),
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn broadcast(&self, data: String, signature: String) -> Result<SuiBroadcastTransaction, Box<dyn Error + Send + Sync>> {
        let transaction = decode_transaction_base64(&data)?;
        let signature = sui_types::UserSignature::from_base64(&signature)?;
        let request = ExecuteTransactionRequest {
            transaction: Some(transaction),
            signatures: vec![GrpcUserSignature::from_sdk(signature)],
            read_mask: Some(FieldMask::from_paths(["digest", "effects.status"])),
        };
        let response: ExecuteTransactionResponse = self.grpc_unary(PATH_EXECUTE_TRANSACTION, request).await?;
        Ok(SuiBroadcastTransaction {
            digest: response
                .transaction
                .and_then(|transaction| transaction.digest)
                .ok_or("missing Sui broadcast transaction digest")?,
        })
    }

    pub(crate) async fn get_multiple_objects(&self, object_ids: Vec<String>) -> Result<Vec<proto::Object>, Box<dyn Error + Send + Sync>> {
        let requests = object_ids
            .into_iter()
            .map(|object_id| Ok(GetObjectRequest::new(&Address::from_str(&object_id)?)))
            .collect::<Result<Vec<_>, Box<dyn Error + Send + Sync>>>()?;
        let request = BatchGetObjectsRequest {
            requests,
            read_mask: Some(FieldMask::from_paths(["object_id", "version", "digest", "owner"])),
        };
        let response: BatchGetObjectsResponse = self.grpc_unary(PATH_BATCH_GET_OBJECTS, request).await?;
        response
            .objects
            .into_iter()
            .map(|result| match result {
                proto::GetObjectResult::Object(object) => Ok(*object),
                proto::GetObjectResult::Error(status) => Err(format!("Sui object gRPC error {}: {}", status.code, status.message).into()),
                proto::GetObjectResult::Unknown => Err("missing Sui object result".into()),
            })
            .collect()
    }

    async fn service_info(&self) -> Result<GetServiceInfoResponse, Box<dyn Error + Send + Sync>> {
        self.grpc_unary(PATH_GET_SERVICE_INFO, GetServiceInfoRequest).await
    }

    pub(super) async fn get_epoch(&self, epoch: Option<u64>, read_mask: Option<FieldMask>) -> Result<proto::Epoch, Box<dyn Error + Send + Sync>> {
        let request = GetEpochRequest { epoch, read_mask };
        let response: GetEpochResponse = self.grpc_unary(PATH_GET_EPOCH, request).await?;
        Ok(response.epoch.ok_or("missing Sui epoch")?)
    }
}

fn decode_transaction_base64(tx_data: &str) -> Result<GrpcTransaction, Box<dyn Error + Send + Sync>> {
    decode_transaction_bytes(&decode_base64(tx_data)?)
}

fn decode_transaction_bytes(tx_data: &[u8]) -> Result<GrpcTransaction, Box<dyn Error + Send + Sync>> {
    let _: sui_types::Transaction = bcs::from_bytes(tx_data)?;
    Ok(GrpcTransaction::from_transaction_bcs(tx_data.to_vec()))
}

fn decode_inspect_transaction_bytes(sender: &str, tx_data: &[u8]) -> Result<GrpcTransaction, Box<dyn Error + Send + Sync>> {
    if let Ok(transaction) = decode_transaction_bytes(tx_data) {
        return Ok(transaction);
    }
    let kind: sui_types::TransactionKind = bcs::from_bytes(tx_data)?;
    Ok(GrpcTransaction::from_kind(proto::TransactionKind::from_sdk(kind)?, sender))
}

fn full_sui_coin_type(coin_type: &str) -> String {
    match coin_type {
        SUI_COIN_TYPE => SUI_COIN_TYPE_FULL.to_string(),
        _ => coin_type.to_string(),
    }
}

fn map_service_chain_identifier(service_info: proto::GetServiceInfoResponse) -> Result<String, Box<dyn Error + Send + Sync>> {
    if service_info.chain.as_deref() == Some("mainnet") {
        return Ok(Chain::Sui.network_id().to_string());
    }

    Ok(service_info.chain_id.ok_or("missing Sui chain id")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_service_chain_identifier() {
        let service_info = proto::GetServiceInfoResponse {
            chain: Some("mainnet".to_string()),
            chain_id: Some("grpc-mainnet-genesis-digest".to_string()),
            ..Default::default()
        };
        assert_eq!(map_service_chain_identifier(service_info).unwrap(), Chain::Sui.network_id());

        let testnet = "sui-testnet-chain-id";
        let service_info = proto::GetServiceInfoResponse {
            chain: Some("testnet".to_string()),
            chain_id: Some(testnet.to_string()),
            ..Default::default()
        };
        assert_eq!(map_service_chain_identifier(service_info).unwrap(), testnet);
    }
}
