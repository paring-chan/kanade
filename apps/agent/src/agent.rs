use crate::config::AgentConfig;
use reqwest::Client;
use std::sync::Arc;

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
        Ok(())
    }
}
