use crate::db::MySQL;
use crate::utils::midgard::MidGard;
use crate::utils::transaction_handler::{TransactionError, TransactionHandler}; // Use the custom error type
use crate::utils::{read_next_page_token_from_file, write_next_page_token_to_file};
use chrono::Utc;

pub async fn fetch_historical_data() -> Result<(), TransactionError> {
    let mysql = MySQL::init().await.map_err(|e| {
        TransactionError::DatabaseError(format!("Error connecting to MySQL: {:?}", e))
    })?;

    let mut next_page_token = read_next_page_token_from_file().unwrap_or_default();

    loop {
        println!("Next Page Token: {}", &next_page_token);

        // Fetch actions using the next page token
        let resp = match MidGard::fetch_actions_with_nextpage(next_page_token.as_str()).await {
            Ok(resp) => resp,
            Err(err) => {
                return Err(TransactionError::ApiError(format!(
                    "Error fetching actions data: {:?}",
                    err
                )));
            }
        };

        if resp.actions.is_empty() {
            break;
        }

        let process_response =
            TransactionHandler::process_and_insert_transaction(&mysql, &resp.actions).await;
        match process_response {
            Ok(_) => {
                next_page_token = resp.meta.nextPageToken.clone();
                if let Err(e) = write_next_page_token_to_file(&next_page_token) {
                    return Err(TransactionError::FileError(format!(
                        "Error writing next page token to file: {:?}",
                        e
                    )));
                }
                println!("Current Token in file: {}", &next_page_token);
            }
            Err(err) => {
                return Err(TransactionError::ProcessingError(format!(
                    "An unexpected error occurred while processing the transaction: {:?}",
                    err
                )));
            }
        }
    }

    Ok(())
}

pub async fn fetch_latest_data(mysql: &MySQL) -> Result<(), TransactionError> {
    let latest_timestamp = match mysql.fetch_latest_timestamp().await {
        Ok(Some(timestamp)) => timestamp,
        Ok(None) => Utc::now().timestamp() as i64,
        Err(err) => {
            return Err(TransactionError::DatabaseError(format!(
                "Error fetching the latest timestamp: {:?}",
                err
            )));
        }
    };

    let mysql_clone = mysql.clone();
    let latest_timestamp_str = latest_timestamp.to_string();

    // Fetch actions with the latest timestamp
    let mut resp = match MidGard::fetch_actions_with_timestamp(&latest_timestamp_str).await {
        Ok(response) => response,
        Err(err) => {
            return Err(TransactionError::ApiError(format!(
                "Error fetching actions with timestamp: {:?}",
                err
            )));
        }
    };

    println!("{:?}", &resp);

    let process_response =
        TransactionHandler::process_and_insert_transaction(&mysql_clone, &resp.actions).await;
    match process_response {
        Ok(_) => (),
        Err(err) => {
            return Err(TransactionError::ProcessingError(format!(
                "Error processing transaction: {:?}",
                err
            )));
        }
    };

    while !resp.actions.is_empty() {
        let prev_page_token = resp.meta.prevPageToken.clone();
        resp = match MidGard::fetch_actions_with_prevpage(prev_page_token.as_str()).await {
            Ok(response) => response,
            Err(err) => {
                return Err(TransactionError::ApiError(format!(
                    "Error fetching previous page actions: {:?}",
                    err
                )));
            }
        };

        let process_response =
            TransactionHandler::process_and_insert_transaction(&mysql_clone, &resp.actions).await;
        match process_response {
            Ok(_) => (),
            Err(err) => {
                return Err(TransactionError::ProcessingError(format!(
                    "Error processing transaction: {:?}",
                    err
                )));
            }
        };
    }

    println!("Latest Data Updated at : {}", latest_timestamp_str);
    Ok(())
}
