use crate::error::{ApiError, Result};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

#[derive(Clone)]
pub struct WikiClient {
    client: reqwest::Client,
    base: String,
}

impl WikiClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("gtw/0.1.0"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        Self {
            client,
            base: "https://en.wikipedia.org".to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        let url = format!("{}/w/api.php", self.base);
        let resp = self
            .client
            .get(&url)
            .query(&[
                ("action", "opensearch"),
                ("search", query),
                ("limit", "10"),
                ("format", "json"),
            ])
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        if resp.status().as_u16() == 429 {
            return Err(ApiError::RateLimited);
        }
        if !resp.status().is_success() {
            return Err(ApiError::Http(resp.status().as_u16()));
        }
        let v: serde_json::Value = resp.json().await.map_err(|e| ApiError::Network(e.to_string()))?;
        let mut out = Vec::new();
        if let Some(arr) = v.as_array().and_then(|a| a.get(1)).and_then(|x| x.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    out.push(s.to_string());
                }
            }
        }
        Ok(out)
    }
}

impl Default for WikiClient {
    fn default() -> Self {
        Self::new()
    }
}
