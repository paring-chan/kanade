use crate::config::AgentConfig;
use crate::reporter::HttpReporter;
use api_types::JobAcquireResponse;
use chrono::Duration;
use job_executor::{Job, JobExecutor, JobStep};
use reqwest::Client;
use std::sync::Arc;
use tokio::time::sleep;

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

    pub async fn run(&self) {
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
                            match serde_json::from_str::<JobAcquireResponse>(&body) {
                                Ok(job) => {
                                    tracing::info!("Acquired job: {:?}", job);
                                    let executor = JobExecutor::new().unwrap();
                                    let reporter = HttpReporter::new(
                                        self.config.api_uri.clone(),
                                        self.client.clone(),
                                    );
                                    let job_to_run = Job {
                                        id: job.id,
                                        image: job.job.image.clone(),
                                        timeout: Duration::minutes(job.job.timeout as i64),
                                        steps: job
                                            .steps
                                            .into_iter()
                                            .map(|s| JobStep {
                                                id: s.id,
                                                name: s.step.name.clone(),
                                                ordering: s.step.ordering,
                                                command: s.step.command.clone(),
                                            })
                                            .collect(),
                                    };
                                    if let Err(e) = executor.run(job_to_run, &reporter).await {
                                        tracing::error!("Failed to run job: {:?}", e);
                                    }
                                }
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

            sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
