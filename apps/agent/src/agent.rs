use crate::reporter::HttpReporter;
use crate::{config::AgentConfig, ws::LogSender};
use api_types::JobAcquireResponse;
use chrono::Duration;
use job_executor::adapter::JobStatusReport;
use job_executor::{Job, JobExecutor, JobStep};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tokio::time::sleep;

pub struct KanadeAgent {
    config: Arc<AgentConfig>,
    log_sender: Arc<LogSender>,
    api_client: Client,
}

impl KanadeAgent {
    pub fn new(config: Arc<AgentConfig>, log_sender: Arc<LogSender>) -> Self {
        Self {
            api_client: Client::builder()
                .default_headers({
                    let mut headers = HeaderMap::new();

                    headers.insert(
                        "x-agent-token",
                        HeaderValue::from_str(config.token.expose_secret()).unwrap(),
                    );

                    headers
                })
                .build()
                .unwrap(),
            config,
            log_sender,
        }
    }

    pub async fn run(&self) {
        tracing::info!("Agent starting...");
        let uri = format!("{}/api/v1/agent/jobs/acquire", self.config.api_uri);

        loop {
            match self.api_client.post(&uri).send().await {
                Ok(response) => {
                    let status = response.status();

                    match status {
                        reqwest::StatusCode::NO_CONTENT => {
                            tracing::debug!("No job acquired");

                            sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                        reqwest::StatusCode::OK => {
                            let body = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Could not read body".to_string());
                            match serde_json::from_str::<JobAcquireResponse>(&body) {
                                Ok(job) => {
                                    tracing::info!("Acquired job: {}", job.id);
                                    let executor = JobExecutor::new().unwrap();
                                    let reporter = HttpReporter::new(
                                        self.config.api_uri.clone(),
                                        self.api_client.clone(),
                                        self.log_sender.clone(),
                                    );
                                    let job_to_run = Job {
                                        id: job.id,
                                        image: job.job.image.clone().unwrap_or_default(),
                                        timeout: Duration::minutes(job.job.timeout as i64),
                                        steps: job
                                            .steps
                                            .into_iter()
                                            .map(|s| JobStep {
                                                id: s.id,
                                                name: s.name.clone(),
                                                ordering: s.ordering,
                                                command: s.command.clone(),
                                                env: s.env,
                                            })
                                            .collect(),
                                        env: job.env,
                                        secrets: job
                                            .secrets
                                            .into_iter()
                                            .map(|(k, v)| (k, v.into()))
                                            .collect(),
                                        ssh_key: SecretString::from(job.ssh_key),
                                    };
                                    if let Err(e) = executor.run(job_to_run, &reporter).await {
                                        tracing::error!("Failed to run job: {:?}", e);
                                        _ = reporter.job_finished(job.id, false).await;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to parse job JSON: {}. Raw body: {}",
                                        e,
                                        body
                                    );
                                    sleep(tokio::time::Duration::from_secs(5)).await;
                                }
                            }
                        }
                        _ => {
                            let body = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Could not read body".to_string());
                            tracing::error!("Unexpected status {}: {}", status, body);
                            sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to acquire job: {}", e);
                }
            }
        }
    }
}
