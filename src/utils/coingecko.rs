use crate::models::{CoinSearchResponse, PriceFetchResponse};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, StatusCode,
};
use std::sync::Arc;
use std::{collections::HashMap, env, error::Error};
use tokio::sync::RwLock;

pub struct CoinGecko {
    client: Client,
    base_url: String,
    coin_id: HashMap<String, String>,
}

impl CoinGecko {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        dotenv().ok();
        let coingecko_base_url = env::var("COINGECKO_BASE_URL").expect("Coingecko BASE URL needed");
        let coingecko_api_key = env::var("COINGECKO_API_KEY").expect("Coingecko API KEY needed");
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-cg-demo-api-key",
            HeaderValue::from_str(&coingecko_api_key)?,
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        let client = Client::builder().default_headers(headers).build()?;
        let coin_id: HashMap<String, String> = HashMap::new();
        Ok(Self {
            client,
            base_url: coingecko_base_url,
            coin_id,
        })
    }

    // Fetch the USD price for a specific coin and date
    pub async fn fetch_usd_price(&self, coin_id: &str, date: &str) -> Result<f64, Box<dyn Error>> {
        let url = format!("{}/coins/{}/history?date={}", self.base_url, coin_id, date);
        let response = self.client.get(&url).send().await?;
        let resp: PriceFetchResponse = response.json().await?;

        Ok(resp.market_data.current_price.usd)
    }

    // Search for a coin by name
    pub async fn search_coin(&self, coin_name: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/search?query={}", self.base_url, coin_name);

        let response = self.client.get(&url).send().await?;
        let resp: CoinSearchResponse = response.json().await?;

        let top_result = resp.coins.get(0);
        match top_result {
            Some(coin) => Ok(coin.id.to_owned()),
            None => Err(format!("Cannot find any coin with name: {}", coin_name).into()),
        }
    }

    pub fn get_coin_id(&self, asset_name: &str) -> Option<String> {
        match self.coin_id.get(asset_name) {
            Some(val) => Some(val.to_string()),
            None => None,
        }
    }

    pub fn add_coin_id(&mut self, coin_name: &str, coin_id: &str) {
        self.coin_id
            .insert(coin_name.to_string(), coin_id.to_string());
    }
}

pub static COINGECKO_INSTANCE: Lazy<Arc<RwLock<CoinGecko>>> = Lazy::new(|| {
    Arc::new(RwLock::new(
        CoinGecko::init().expect("Failed to initialize CoinGecko client"),
    ))
});
