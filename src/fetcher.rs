use crate::db::MySQL;
use crate::utils::midgard::MidGard;
use crate::utils::transaction_handler::TransactionHandler;
use crate::utils::{read_next_page_token_from_file, write_next_page_token_to_file};
use chrono::Utc;
use std::error::Error;
use std::time::Duration;
use tokio::time;

pub async fn fetch_historical_data(mysql: &MySQL) -> Result<(), Box<dyn Error>> {
    let mut next_page_token = read_next_page_token_from_file().unwrap_or_default();
    loop {
        println!("Next Page Token: {}", &next_page_token);

        match MidGard::fetch_actions_with_nextpage(next_page_token.as_str()).await {
            Ok(resp) => {
                if resp.actions.is_empty() {
                    break;
                }
                let process_response =
                    TransactionHandler::process_and_insert_transaction(&mysql, &resp.actions).await;

                match process_response {
                    Ok(_) => {
                        next_page_token = resp.meta.nextPageToken.clone();
                        if let Err(e) = write_next_page_token_to_file(&next_page_token) {
                            println!("Error writing next page token to file: {:?}", e);
                        } else {
                            println!("Current Token in file: {}", &next_page_token);
                        }
                    }
                    Err(err) => {
                        println!("An unexpected error occurred: {:?}", err);
                    }
                }
            }
            Err(err) => {
                println!("Error fetching actions data: {:?}", err);
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    Ok(())
}

pub async fn fetch_latest_data(mysql: &MySQL) -> Result<(), Box<dyn Error>> {
    let latest_timestamp = match mysql.fetch_latest_timestamp().await {
        Ok(Some(timestamp)) => timestamp,
        Ok(None) => Utc::now().timestamp() as i64,
        Err(err) => {
            println!("Error Fetching Last Timestamp {:?}", err);
            Utc::now().timestamp() as i64
        }
    };

    let mysql_clone = mysql.clone();

    let latest_timestamp_str = latest_timestamp.to_string();

    let mut resp = MidGard::fetch_actions_with_timestamp(&latest_timestamp_str).await?;
    println!("{:?}", &resp);
    TransactionHandler::process_and_insert_transaction(&mysql_clone, &resp.actions).await?;

    while resp.actions.len() != 0 {
        let prev_page_token = resp.meta.prevPageToken;
        resp = MidGard::fetch_actions_with_prevpage(prev_page_token.as_str()).await?;
        TransactionHandler::process_and_insert_transaction(&mysql_clone, &resp.actions).await?;
    }

    println!("Latest Data Updated at : {}", latest_timestamp_str);
    Ok(())
}
