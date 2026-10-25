use crate::api::error::{ApiError, Result};
#[allow(dead_code)]
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WikiClient {
    client: reqwest::Client,
    base: String,
}

impl WikiClient {
    #[allow(dead_code)]
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
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let mut out = Vec::new();
        if let Some(arr) = v
            .as_array()
            .and_then(|a| a.get(1))
            .and_then(|x| x.as_array())
        {
            for item in arr {
                if let Some(s) = item.as_str() {
                    out.push(s.to_string());
                }
            }
        }
        Ok(out)
    }

    pub async fn get_article_html(
        &self,
        title: &str,
    ) -> Result<(String, Option<String>, Option<String>)> {
        let url = format!("{}/w/api.php", self.base);
        let resp = self
            .client
            .get(&url)
            .query(&[
                ("action", "parse"),
                ("page", title),
                ("format", "json"),
                ("prop", "text|revisions"),
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
        let etag = resp
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let last_modified = resp
            .headers()
            .get("last-modified")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let html = v["parse"]["text"]["*"].as_str().unwrap_or("").to_string();
        Ok((html, etag, last_modified))
    }
}

impl Default for WikiClient {
    fn default() -> Self {
        Self::new()
    }
}
