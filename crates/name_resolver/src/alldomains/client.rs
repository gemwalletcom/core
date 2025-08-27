use std::{error::Error, str::FromStr};

use async_trait::async_trait;
use base64::Engine;
use serde_json::{self, json};
use sha2::{Digest, Sha256};

use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use gem_solana::pubkey::Pubkey;
use primitives::{chain::Chain, name::NameProvider};

use crate::client::NameClient;

use super::model::NameRecordHeader;

const ANS_PROGRAM_ID: &str = "ALTNSZ46uaAUU7XUV6awvdorLGqAsPwa9shm7h4uP2FK";
const TLD_HOUSE_PROGRAM_ID: &str = "TLDHkysf5pCnKsVA4gXpNvmy7psXLPEu4LAdDJthT9S";
const NAME_HOUSE_PROGRAM_ID: &str = "NH3uX6FtVE2fNREAioP7hm5RaozotZxeL6khU1EHx51";
const ROOT_ANS_PUBLIC_KEY: &str = "3mX9b4AZaQehNoQGfckVcmgmA6bkBoFcbLj9RMmMyNcU";
const HASH_PREFIX: &str = "ALT Name Service";
const TLD_HOUSE_PREFIX: &str = "tld_house";
const NAME_HOUSE_PREFIX: &str = "name_house";
const NFT_RECORD_PREFIX: &str = "nft_record";

pub struct AllDomainsClient {
    client: JsonRpcClient<ReqwestClient>,
}

impl AllDomainsClient {
    pub fn new(url: String) -> Self {
        Self {
            client: JsonRpcClient::new(ReqwestClient::new(url, reqwest::Client::new())),
        }
    }

    async fn get_hashed_name(&self, name: &str) -> Result<[u8; 32], Box<dyn Error + Send + Sync>> {
        let mut hasher = Sha256::new();
        hasher.update(HASH_PREFIX.as_bytes());
        hasher.update(name.as_bytes());
        Ok(hasher.finalize().into())
    }

    fn get_name_account_key_with_bump(
        &self,
        hashed_name: &[u8; 32],
        name_class: Option<Pubkey>,
        parent_name: Option<Pubkey>,
    ) -> Result<(Pubkey, u8), Box<dyn Error + Send + Sync>> {
        let mut seeds = Vec::new();
        seeds.push(hashed_name.as_ref());

        let default_pubkey = Pubkey::from([0u8; 32]);
        let name_class_key = name_class.unwrap_or(default_pubkey.clone());
        let parent_name_key = parent_name.unwrap_or(default_pubkey);
        seeds.push(name_class_key.as_ref());
        seeds.push(parent_name_key.as_ref());

        let ans_program_id = Pubkey::from_str(ANS_PROGRAM_ID)?;
        if let Some((pda, bump)) = Pubkey::try_find_program_address(&seeds, &ans_program_id) {
            Ok((pda, bump))
        } else {
            Err("Failed to derive PDA".into())
        }
    }

    fn find_tld_house(&self, tld_string: &str) -> Result<(Pubkey, u8), Box<dyn Error + Send + Sync>> {
        let tld_lower = tld_string.to_lowercase();
        let seeds = &[TLD_HOUSE_PREFIX.as_bytes(), tld_lower.as_bytes()];

        let tld_house_program_id = Pubkey::from_str(TLD_HOUSE_PROGRAM_ID)?;
        if let Some((pda, bump)) = Pubkey::try_find_program_address(seeds, &tld_house_program_id) {
            Ok((pda, bump))
        } else {
            Err("Failed to derive TLD house PDA".into())
        }
    }

    fn find_name_house(&self, tld_house: Pubkey) -> Result<(Pubkey, u8), Box<dyn Error + Send + Sync>> {
        let seeds = &[NAME_HOUSE_PREFIX.as_bytes(), tld_house.as_ref()];
        let name_house_program_id = Pubkey::from_str(NAME_HOUSE_PROGRAM_ID)?;
        if let Some((pda, bump)) = Pubkey::try_find_program_address(seeds, &name_house_program_id) {
            Ok((pda, bump))
        } else {
            Err("Failed to derive name house PDA".into())
        }
    }

    fn find_nft_record(&self, name_account: Pubkey, name_house_account: Pubkey) -> Result<(Pubkey, u8), Box<dyn Error + Send + Sync>> {
        let seeds = &[NFT_RECORD_PREFIX.as_bytes(), name_house_account.as_ref(), name_account.as_ref()];
        let name_house_program_id = Pubkey::from_str(NAME_HOUSE_PROGRAM_ID)?;
        if let Some((pda, bump)) = Pubkey::try_find_program_address(seeds, &name_house_program_id) {
            Ok((pda, bump))
        } else {
            Err("Failed to derive NFT record PDA".into())
        }
    }

    async fn get_origin_name_account_key(&self) -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
        let root_pubkey = Pubkey::from_str(ROOT_ANS_PUBLIC_KEY)?;
        Ok(root_pubkey)
    }

    async fn get_name_owner(&self, name_account_key: Pubkey, tld_house: Option<Pubkey>) -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
        let response: serde_json::Value = self
            .client
            .call(
                "getAccountInfo",
                vec![json!(name_account_key.to_string()), json!({"encoding": "base64", "commitment": "confirmed"})],
            )
            .await?;

        let account_value = response.get("value").ok_or("Invalid response format")?;
        if account_value.is_null() {
            return Err("Account not found or domain does not exist".into());
        }

        let account_obj = account_value.as_object().ok_or("Invalid account data format")?;
        let data_array = account_obj.get("data").and_then(|d| d.as_array()).ok_or("No data field in account")?;

        if data_array.is_empty() {
            return Err("Empty account data".into());
        }

        let base64_data = data_array[0].as_str().ok_or("Invalid base64 data format")?;
        let data = base64::engine::general_purpose::STANDARD.decode(base64_data)?;
        let name_record = NameRecordHeader::try_from_slice(&data)?;

        if !name_record.is_valid() {
            return Err("Name record is not valid or expired".into());
        }

        let owner = name_record.owner;

        if let Some(tld_house) = tld_house {
            let (name_house, _) = self.find_name_house(tld_house)?;
            let (nft_record, _) = self.find_nft_record(name_account_key, name_house)?;

            if owner == nft_record {
                return Err("NFT owner resolution is not supported".into());
            }
        }

        Ok(owner)
    }
}

#[async_trait]
impl NameClient for AllDomainsClient {
    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Split domain.tld
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid domain format".into());
        }
        let domain = parts[0];
        let tld = parts[1];

        let name_origin_tld_key = self.get_origin_name_account_key().await?;

        let tld_name = format!(".{tld}");
        let parent_hashed_name = self.get_hashed_name(&tld_name).await?;
        let (parent_account_key, _) = self.get_name_account_key_with_bump(&parent_hashed_name, None, Some(name_origin_tld_key))?;

        let domain_hashed_name = self.get_hashed_name(domain).await?;
        let (domain_account_key, _) = self.get_name_account_key_with_bump(&domain_hashed_name, None, Some(parent_account_key))?;

        let (tld_house, _) = self.find_tld_house(&tld_name)?;
        let name_owner = self.get_name_owner(domain_account_key, Some(tld_house)).await?;

        Ok(name_owner.to_string())
    }

    fn provider(&self) -> NameProvider {
        NameProvider::AllDomains
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["skr", "saga", "poor", "bonk", "solana"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Solana]
    }
}
