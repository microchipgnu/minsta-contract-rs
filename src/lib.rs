use near_sdk::{
    borsh::{ self, BorshDeserialize, BorshSerialize },
    collections::LookupMap,
    env,
    near_bindgen,
    Promise,
    AccountId,
};
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    title: Option<String>,
    description: Option<String>,
    media: Option<String>,
    media_hash: Option<String>,
    copies: Option<i32>,
    expires_at: Option<i64>,
    starts_at: Option<i64>,
    extra: Option<String>,
    reference: Option<String>,
    reference_hash: Option<String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MinstaProxyMinter {
    latest_minters: LookupMap<AccountId, AccountId>,
}

impl Default for MinstaProxyMinter {
    fn default() -> Self {
        Self {
            latest_minters: LookupMap::new(b"latest_minters".to_vec()),
        }
    }
}

#[near_bindgen]
impl MinstaProxyMinter {
    pub fn mint(&mut self, metadata: String, nft_contract_id: AccountId) -> Promise {
        let parsed_metadata: Result<Metadata, serde_json::Error> = serde_json::from_str(&metadata);

        match parsed_metadata {
            Ok(parsed) => {
                let minter_id = env::predecessor_account_id();
                let prev_minter_id = self.latest_minters
                    .get(&nft_contract_id)
                    .unwrap_or(minter_id.clone());
                let mut royalty_args = std::collections::HashMap::new();
                royalty_args.insert(minter_id.clone(), 10000);

                let args =
                    serde_json::json!({
                    "owner_id": prev_minter_id,
                    "metadata": parsed,
                    "num_to_mint": 1,
                    "royalty_args": {
                        "split_between": royalty_args,
                        "percentage": 1000
                    },
                    "split_owners": serde_json::Value::Null
                });

                Promise::new(nft_contract_id.clone())
                    .function_call(
                        "nft_batch_mint".to_string(),
                        args.to_string().into_bytes(),
                        (0 as u128).into(),
                        near_sdk::Gas(100_000_000_000_000)
                    )
                    .then(
                        Promise::new(env::current_account_id()).function_call(
                            "cb_mint".to_string(),
                            serde_json::json!({
                                "latest_minter_id": minter_id,
                                "nft_contract_id": nft_contract_id.clone()
                            })
                                .to_string()
                                .into_bytes(),
                            (0 as u128).into(),
                            near_sdk::Gas(50_000_000_000_000)
                        )
                    )
            }
            Err(_) => env::panic(b"Failed to parse metadata"),
        }
    }

    pub fn cb_mint(&mut self, latest_minter_id: AccountId, nft_contract_id: AccountId) -> bool {
        if env::promise_results_count() == 1 {
            self.latest_minters.insert(&nft_contract_id, &latest_minter_id);
            true
        } else {
            false
        }
    }

    pub fn get_latest_minter(&self, nft_contract_id: AccountId) -> Option<AccountId> {
        self.latest_minters.get(&nft_contract_id)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests here.
}
