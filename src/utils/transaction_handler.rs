use super::{calculate_transaction_amount, coingecko::COINGECKO_INSTANCE};
use crate::{
    db::MySQL,
    models::actions_model::{SwapTransaction, SwapTransactionFromatted, TransactionData},
    utils::{
        asset_name_from_pool, coin_name_from_pool, convert_nano_to_sec, convert_to_standard_unit,
        format_epoch_timestamp, parse_f64, parse_u64,
    },
};
use std::error::Error;

pub struct TransactionHandler;

impl TransactionHandler {
    pub async fn parse_data(
        &self,
        info: &TransactionData,
        swap_date: &str,
    ) -> Result<(String, f64, f64, String), Box<dyn Error>> {
        let in_coin = info.coins.get(0).ok_or("Missing in_coin")?;
        let coin_name = coin_name_from_pool(&in_coin.asset).ok_or("Error parsing in_asset")?;
        let in_amount = parse_f64(&in_coin.amount)?;
        let in_amount_usd = self
            .convert_amount_to_usd(&coin_name, swap_date, in_amount)
            .await?;
        let in_asset = asset_name_from_pool(&in_coin.asset).ok_or("Error parsing asset name")?;
        let in_address = info.address.clone();
        Ok((
            in_asset,
            convert_to_standard_unit(in_amount, 8),
            in_amount_usd,
            in_address,
        ))
    }

    pub async fn convert_amount_to_usd(
        &self,
        asset_name: &str,
        date: &str,
        amount: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let mut coingecko = COINGECKO_INSTANCE.write().await;
        let coin_id = match coingecko.get_coin_id(&asset_name) {
            Some(coin_id) => coin_id,
            None => {
                let coin_id = coingecko.search_coin(asset_name).await?;
                coingecko.add_coin_id(&asset_name, &coin_id);
                coin_id
            }
        };
        let price_on_date = coingecko
            .fetch_usd_price(coin_id.as_str().as_ref(), date)
            .await?;
        let amount = convert_to_standard_unit(amount, 8);
        let t_amount = calculate_transaction_amount(amount.clone(), price_on_date);
        Ok((t_amount * 100.0).round() / 100.0)
    }

    pub async fn parse_transaction(
        swap: &SwapTransaction,
    ) -> Result<SwapTransactionFromatted, Box<dyn Error>> {
        if &swap.status != "success" {
            return Err("Swap is in Progress".into());
        }

        // Parse swap_date & swap_time
        let (swap_date, swap_time) = format_epoch_timestamp(&swap.date)?;
        let epoc_timestamp = parse_f64(convert_nano_to_sec(&swap.date).as_str()).unwrap() as i64;

        println!("Current Progress Date : {}", &swap_date);

        // Parse tx_id from in_data
        let tx_id = swap
            .in_data
            .get(0)
            .and_then(|data| data.txID.clone())
            .ok_or("Missing or invalid TxId")?;

        let handler = TransactionHandler;

        // Parse In Data
        let in_data = swap.in_data.get(0).ok_or("No In Data Found")?;
        let (in_asset, in_amount, in_amount_usd, in_address) =
            handler.parse_data(in_data, &swap_date).await?;

        let mut out_data = swap.out_data.clone();
        out_data.reverse();

        // Parse Out Data
        let out_data_1 = out_data.get(0).ok_or("No Out Data Found")?;
        let (out_asset_1, out_amount_1, out_amount_1_usd, out_address_1) =
            handler.parse_data(out_data_1, &swap_date).await?;

        let (out_asset_2, out_amount_2, out_amount_2_usd, out_address_2) = if let Some(val) =
            out_data.get(1)
        {
            let (asset, amount, amount_usd, address) = handler.parse_data(val, &swap_date).await?;
            (Some(asset), Some(amount), Some(amount_usd), Some(address))
        } else {
            (None, None, None, None)
        };

        Ok(SwapTransactionFromatted {
            timestamp: epoc_timestamp,
            date: swap_date,
            time: swap_time,
            in_asset,
            in_amount,
            in_amount_usd,
            out_asset_1,
            out_amount_1,
            out_amount_1_usd,
            in_address,
            out_address_1,
            tx_id,
            out_asset_2,
            out_amount_2,
            out_amount_2_usd,
            out_address_2,
        })
    }

    pub async fn process_and_insert_transaction(
        mysql: &MySQL,
        actions: &Vec<SwapTransaction>,
    ) -> Result<(), Box<dyn Error>> {
        for swap in actions {
            let transaction_info = TransactionHandler::parse_transaction(&swap).await;

            let transaction_info = match transaction_info {
                Ok(val) => val,
                Err(err) => {
                    println!("Error parsing transaction: {:?}", err);
                    continue;
                }
            };
            if let Err(err) = mysql.insert_new_record(transaction_info.clone()).await {
                println!("Error during insertion: {:?}", err);
            } else {
                println!("Insertion Successful for Id : {}", &transaction_info.tx_id);
            }
        }

        Ok(())
    }
}
