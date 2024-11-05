use crate::models::actions_model::ActionsFetchResponse;
pub struct MidGard;

impl MidGard {
    pub async fn fetch_actions_with_nextpage(
        next_page_token: &str,
    ) -> Result<ActionsFetchResponse, reqwest::Error> {
        let url = if next_page_token.is_empty() {
            "https://vanaheimex.com/actions?type=swap&asset=notrade".to_string()
        } else {
            format!(
                "https://vanaheimex.com/actions?type=swap&asset=notrade&nextPageToken={}",
                next_page_token
            )
        };
        println!("Fetching URL: {}", &url);
        let response = reqwest::get(&url).await?;
        let resp: ActionsFetchResponse = response.json().await?;
        Ok(resp)
    }

    pub async fn fetch_actions_with_prevpage(
        prev_page_token: &str,
    ) -> Result<ActionsFetchResponse, reqwest::Error> {
        let url = format!(
            "https://vanaheimex.com/actions?type=swap&asset=notrade&prevPageToken={}",
            prev_page_token
        );
        println!("Fetching URL: {}", &url);
        let response = reqwest::get(&url).await?;
        let resp: ActionsFetchResponse = response.json().await?;
        Ok(resp)
    }

    pub async fn fetch_actions_with_timestamp(
        timestamp: &str,
    ) -> Result<ActionsFetchResponse, reqwest::Error> {
        let url = format!(
            "https://vanaheimex.com/actions?type=swap&asset=notrade&fromTimestamp={}",
            timestamp
        );
        println!("Fetching URL: {}", &url);
        let response = reqwest::get(&url).await?;
        println!("{:?}", response);
        let resp: ActionsFetchResponse = response.json().await?;
        println!("{:?}", resp);
        Ok(resp)
    }
}
