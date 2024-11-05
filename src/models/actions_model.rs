#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapCoin {
    pub amount: String,
    pub asset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionData {
    pub address: String,
    pub coins: Vec<SwapCoin>,
    pub txID: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapTransaction {
    pub date: String,
    #[serde(rename = "in")]
    pub in_data: Vec<TransactionData>,
    #[serde(rename = "out")]
    pub out_data: Vec<TransactionData>,
    pub pools: Vec<String>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionsFetchMeta {
    pub nextPageToken: String,
    pub prevPageToken: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionsFetchResponse {
    pub actions: Vec<SwapTransaction>,
    pub meta: ActionsFetchMeta,
}

#[derive(Debug, Clone)]
pub struct SwapTransactionFromatted {
    pub timestamp: String,
    pub date: String,
    pub time: String,
    pub in_asset: String,
    pub in_amount: f64,
    pub in_amount_usd: f64,
    pub out_asset_1: String,
    pub out_amount_1: f64,
    pub out_amount_1_usd: f64,
    pub in_address: String,
    pub out_address_1: String,
    pub tx_id: String,
    pub out_asset_2: Option<String>,
    pub out_amount_2: Option<f64>,
    pub out_amount_2_usd: Option<f64>,
    pub out_address_2: Option<String>,
}
