use dotenv::dotenv;
use sqlx::mysql::MySqlPool;
use std::{env, error::Error};

use crate::{
    models::actions_model::SwapTransactionFromatted,
    utils::{format_date_for_sql, parse_u64},
};

#[derive(Clone)]
pub struct MySQL {
    pub pool: MySqlPool,
}

impl MySQL {
    pub async fn init() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = MySqlPool::connect(&database_url)
            .await
            .expect("Error Connecting to MySQL");
        println!("Connected to MySQL");
        MySQL { pool }
    }

    pub async fn insert_new_record(
        &self,
        record: SwapTransactionFromatted,
    ) -> Result<(), Box<dyn Error>> {
        // Parsing and formatting the data
        let timestamp = parse_u64(&record.timestamp).unwrap();
        let date = format_date_for_sql(&record.date).unwrap();
        let time = record.time.clone();

        // Executing the insert query
        sqlx::query!(
            r#"
            INSERT INTO swap_history (timestamp, date, time, tx_id, in_asset, in_amount, in_amount_usd, in_address, out_asset_1, out_amount_1, out_amount_1_usd, out_address_1, out_asset_2, out_amount_2, out_amount_2_usd, out_address_2)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            timestamp,
            date,
            time,
            record.tx_id,
            record.in_asset,
            record.in_amount,
            record.in_amount_usd,
            record.in_address,
            record.out_asset_1,
            record.out_amount_1,
            record.out_amount_1_usd,
            record.out_address_1,
            record.out_asset_2,
            record.out_amount_2,
            record.out_amount_2_usd,
            record.out_address_2
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_latest_timestamp(
        &self,
    ) -> Result<Option<i64>, Box<dyn Error + Send + Sync>> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT MAX(timestamp) as "timestamp: i64"
            FROM swap_history
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }
}
