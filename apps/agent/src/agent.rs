use crate::config::AgentConfig;
use api_types::PipelineJobRunResponse;
use reqwest::Client;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

pub struct KanadeAgent {
    config: Arc<AgentConfig>,
    client: Client,
}

impl KanadeAgent {
    pub fn new(config: Arc<AgentConfig>) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("Agent starting...");
        let uri = format!("{}/api/v1/agent/jobs/acquire", self.config.api_uri);

        loop {
            match self.client.post(&uri).send().await {
                Ok(response) => {
                    let status = response.status();

                    match status {
                        reqwest::StatusCode::NO_CONTENT => {
                            tracing::debug!("No job acquired");
                        }
                        reqwest::StatusCode::OK => {
                            let body = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Could not read body".to_string());
                            match serde_json::from_str::<PipelineJobRunResponse>(&body) {
                                Ok(job) => tracing::info!("Acquired job: {:?}", job),
                                Err(e) => tracing::error!(
                                    "Failed to parse job JSON: {}. Raw body: {}",
                                    e,
                                    body
                                ),
                            }
                        }
                        _ => {
                            let body = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Could not read body".to_string());
                            tracing::error!("Unexpected status {}: {}", status, body);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to acquire job: {}", e);
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}
