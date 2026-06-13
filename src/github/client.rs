use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use reqwest::StatusCode;
use reqwest::header::{ETAG, HeaderMap, HeaderValue, IF_NONE_MATCH, LINK};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use super::events::GitHubEvent;

pub struct GithubClient {
    http: reqwest::Client,
    token: Option<String>,
    etag_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github+json"),
        );
        default_headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let mut builder = reqwest::Client::builder()
            .default_headers(default_headers)
            .user_agent("gh-monitor3");

        if let Some(ref t) = token {
            let mut auth_headers = HeaderMap::new();
            auth_headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {t}"))
                    .map_err(|e| anyhow::anyhow!("Invalid token header: {e}"))?,
            );
            builder = builder.default_headers(auth_headers);
        }

        let http = builder.build()?;

        Ok(Self {
            http,
            token,
            etag_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_gh_cli_token() -> Option<String> {
        std::process::Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub async fn whoami(&self) -> Result<String> {
        let resp = self.http.get("https://api.github.com/user").send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("whoami failed: {}", resp.status());
        }
        let user: serde_json::Value = resp.json().await?;
        Ok(user["login"].as_str().unwrap_or("unknown").to_string())
    }

    pub async fn list_repo_events(&self, owner: &str, repo: &str) -> Result<Vec<GitHubEvent>> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/events");
        self.fetch_events_paginated(&url).await
    }

    pub async fn list_org_events(&self, org: &str) -> Result<Vec<GitHubEvent>> {
        let url = format!("https://api.github.com/orgs/{org}/events");
        self.fetch_events_paginated(&url).await
    }

    async fn fetch_events_paginated(&self, base_url: &str) -> Result<Vec<GitHubEvent>> {
        let mut all_events = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut url = Some(base_url.to_string());

        while let Some(current_url) = url.take() {
            let (events, next_url, _etag) = self.fetch_single_page(&current_url).await?;

            for event in events {
                if seen_ids.insert(event.id.clone()) {
                    all_events.push(event);
                }
            }

            if let Some(next) = next_url {
                url = Some(next);
            }
        }

        Ok(all_events)
    }

    async fn fetch_single_page(
        &self,
        url: &str,
    ) -> Result<(Vec<GitHubEvent>, Option<String>, Option<String>)> {
        let mut req = self.http.get(url);

        {
            let cache = self.etag_cache.lock().await;
            if let Some(etag) = cache.get(url) {
                req = req.header(IF_NONE_MATCH, etag.clone());
            }
        }

        let resp = req.send().await?;

        if resp.status() == StatusCode::NOT_MODIFIED {
            debug!("ETag cache hit for {url}");
            return Ok((Vec::new(), None, None));
        }

        if !resp.status().is_success() {
            warn!("GitHub API error for {url}: {}", resp.status());
            return Ok((Vec::new(), None, None));
        }

        let response_etag = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let next_url = extract_next_link(resp.headers());

        let body = resp.text().await?;

        if let Some(ref etag) = response_etag {
            let mut cache = self.etag_cache.lock().await;
            cache.insert(url.to_string(), etag.clone());
        }

        let gh_events: Vec<octocrab::models::events::Event> = match serde_json::from_str(&body) {
            Ok(events) => events,
            Err(e) => {
                warn!("Failed to parse events from {url}: {e}");
                return Ok((Vec::new(), next_url, response_etag));
            }
        };

        let events: Vec<GitHubEvent> = gh_events.into_iter().map(GitHubEvent::from).collect();

        Ok((events, next_url, response_etag))
    }
}

pub(crate) fn extract_next_link(headers: &HeaderMap) -> Option<String> {
    let link_header = headers.get(LINK)?.to_str().ok()?;
    for part in link_header.split(',') {
        let trimmed = part.trim();
        if trimmed.contains(r#"rel="next""#) {
            let url = trimmed.split('<').nth(1)?.split('>').next()?;
            return Some(url.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderValue, LINK};

    #[test]
    fn new_no_token() {
        let client = GithubClient::new(None).unwrap();
        assert!(!client.has_token());
    }

    #[test]
    fn new_with_token() {
        let client = GithubClient::new(Some("ghp_test123".to_string())).unwrap();
        assert!(client.has_token());
    }

    #[test]
    fn has_token_false() {
        let client = GithubClient::new(None).unwrap();
        assert!(!client.has_token());
    }

    #[test]
    fn has_token_true() {
        let client = GithubClient::new(Some("token".to_string())).unwrap();
        assert!(client.has_token());
    }

    #[test]
    fn extract_next_link_parses_next() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            HeaderValue::from_static(
                r#"<https://api.github.com/repos/owner/repo/events?page=2>; rel="next""#,
            ),
        );
        let result = extract_next_link(&headers);
        assert_eq!(
            result,
            Some("https://api.github.com/repos/owner/repo/events?page=2".to_string())
        );
    }

    #[test]
    fn extract_next_link_missing_header() {
        let headers = HeaderMap::new();
        assert_eq!(extract_next_link(&headers), None);
    }

    #[test]
    fn extract_next_link_no_next_rel() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            HeaderValue::from_static(
                r#"<https://api.github.com/repos/owner/repo/events?page=1>; rel="prev""#,
            ),
        );
        assert_eq!(extract_next_link(&headers), None);
    }

    #[test]
    fn extract_next_link_multiple_rels() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            HeaderValue::from_static(
                r#"<https://api.github.com/repos/owner/repo/events?page=1>; rel="prev", <https://api.github.com/repos/owner/repo/events?page=3>; rel="next""#,
            ),
        );
        let result = extract_next_link(&headers);
        assert_eq!(
            result,
            Some("https://api.github.com/repos/owner/repo/events?page=3".to_string())
        );
    }

    #[test]
    fn extract_next_link_only_last_rel() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            HeaderValue::from_static(
                r#"<https://api.github.com/repos/owner/repo/events?page=5>; rel="last""#,
            ),
        );
        assert_eq!(extract_next_link(&headers), None);
    }
}
