use crate::config::AgentConfig;
use reqwest::Client;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

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
                    tracing::info!("Job acquisition attempt: status={}", status);
                    
                    if status.is_success() {
                        // Handle successful job acquisition if needed in the future
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
