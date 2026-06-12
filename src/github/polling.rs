use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time;
use tracing::{error, info, warn};

use crate::config::Config;

use super::client::GithubClient;
use super::events::GitHubEvent;

pub struct Poller {
    client: Arc<GithubClient>,
    repos: Vec<(String, String)>,
    orgs: Vec<String>,
    interval: Duration,
    event_tx: mpsc::Sender<Vec<GitHubEvent>>,
}

impl Poller {
    pub fn new(config: &Config, event_tx: mpsc::Sender<Vec<GitHubEvent>>) -> Self {
        let client = Arc::new(
            GithubClient::new(config.github_token.clone())
                .expect("Failed to create GitHub client"),
        );

        let repos: Vec<(String, String)> = config
            .repos
            .iter()
            .map(|r| (r.owner.clone(), r.name.clone()))
            .collect();

        Self {
            client,
            repos,
            orgs: config.orgs.clone(),
            interval: Duration::from_secs(config.poll_interval_secs),
            event_tx,
        }
    }

    pub async fn run(&self) {
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut ticker = time::interval(self.interval);
        ticker.tick().await;

        info!(
            "Starting poller: {} repos, {} orgs, interval {:?}",
            self.repos.len(),
            self.orgs.len(),
            self.interval,
        );

        loop {
            self.poll_cycle(&mut seen_ids).await;
            ticker.tick().await;
        }
    }

    async fn poll_cycle(&self, seen_ids: &mut HashSet<String>) {
        for (owner, repo) in &self.repos {
            match self.client.list_repo_events(owner, repo).await {
                Ok(events) => {
                    let new_events: Vec<GitHubEvent> = events
                        .into_iter()
                        .filter(|e| seen_ids.insert(e.id.clone()))
                        .collect();

                    if !new_events.is_empty() {
                        info!(
                            "Repo {}/{}: {} new events",
                            owner,
                            repo,
                            new_events.len()
                        );
                        if self.event_tx.send(new_events).await.is_err() {
                            warn!("Event receiver dropped, stopping poller");
                            return;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch events for {owner}/{repo}: {e}");
                }
            }
        }

        for org in &self.orgs {
            match self.client.list_org_events(org).await {
                Ok(events) => {
                    let new_events: Vec<GitHubEvent> = events
                        .into_iter()
                        .filter(|e| seen_ids.insert(e.id.clone()))
                        .collect();

                    if !new_events.is_empty() {
                        info!("Org {org}: {} new events", new_events.len());
                        if self.event_tx.send(new_events).await.is_err() {
                            warn!("Event receiver dropped, stopping poller");
                            return;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch events for org {org}: {e}");
                }
            }
        }

        trim_seen_ids(seen_ids);
    }
}

fn trim_seen_ids(seen: &mut HashSet<String>) {
    const MAX_SEEN: usize = 10_000;
    if seen.len() > MAX_SEEN {
        let excess = seen.len() - MAX_SEEN;
        let to_remove: Vec<String> = seen.iter().take(excess).cloned().collect();
        for id in to_remove {
            seen.remove(&id);
        }
    }
}
