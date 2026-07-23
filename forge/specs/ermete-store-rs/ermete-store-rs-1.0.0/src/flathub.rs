use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlathubApp {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(rename = "iconDesktopUrl")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    pub hits: Vec<FlathubApp>,
}

pub struct FlathubClient {
    client: Client,
}

impl FlathubClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Search Flathub for an app by query string.
    pub async fn search(&self, query: &str) -> Result<Vec<FlathubApp>> {
        let url = format!("https://flathub.org/api/v2/search/{}", query);
        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            let search_resp: SearchResponse = response.json().await?;
            Ok(search_resp.hits)
        } else {
            Ok(vec![])
        }
    }

    /// Fetch detailed metadata for a specific app ID.
    pub async fn get_app_details(&self, app_id: &str) -> Result<String> {
        let url = format!("https://flathub.org/api/v1/apps/{}", app_id);
        let text = self.client.get(&url).send().await?.text().await?;
        Ok(text) // We just pass the JSON string directly to the frontend
    }
}
