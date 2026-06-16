use std::sync::Arc;

use api_types::{AgentLogKind, AgentLogMessage, JobFinishRequest, StepFinishRequest};
use job_executor::adapter::{JobStatusReport, LogLine};
use reqwest::Client;
use tracing::info;
use uuid::Uuid;

use crate::ws::LogSender;

#[derive(Debug)]
pub struct ReporterError(pub anyhow::Error);

impl std::error::Error for ReporterError {}

impl std::fmt::Display for ReporterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct HttpReporter {
    base_url: String,
    client: Client,
    log_sender: Arc<LogSender>,
}

impl HttpReporter {
    pub fn new(base_url: String, client: Client, log_sender: Arc<LogSender>) -> Self {
        Self {
            base_url,
            client,
            log_sender,
        }
    }
}

impl JobStatusReport for HttpReporter {
    type Error = ReporterError;

    async fn step_started(
        &self,
        job_id: Uuid,
        step_id: Uuid,
        step_name: &str,
    ) -> Result<(), Self::Error> {
        info!(step_id = %step_id, step_name = %step_name, "Step started");
        let url = format!("{}/api/v1/agent/steps/{}/started", self.base_url, step_id);
        self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| ReporterError(e.into()))?;
        Ok(())
    }

    async fn step_finished(
        &self,
        job_id: Uuid,
        step_id: Uuid,
        exit_code: i32,
    ) -> Result<(), Self::Error> {
        info!(step_id = %step_id, exit_code = %exit_code, "Step finished");
        let url = format!("{}/api/v1/agent/steps/{}/finish", self.base_url, step_id);
        let request = StepFinishRequest {
            success: exit_code == 0,
            exit_code,
        };
        self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ReporterError(e.into()))?;
        Ok(())
    }

    async fn step_log(
        &self,
        job_id: Uuid,
        step_id: Uuid,
        line: LogLine,
    ) -> Result<(), Self::Error> {
        debug!(step_id = %step_id, line = ?line, "Step log");

        self.log_sender
            .sender
            .send(match line {
                LogLine::StdIn(_) => return Ok(()),
                LogLine::StdOut(stdout) => AgentLogMessage::Log {
                    job_id,
                    step_id,
                    kind: AgentLogKind::Stdout,
                    content: stdout,
                },
                LogLine::StdErr(stderr) => AgentLogMessage::Log {
                    job_id,
                    step_id,
                    kind: AgentLogKind::Stderr,
                    content: stderr,
                },
            })
            .await
            .map_err(|e| ReporterError(e.into()))?;

        Ok(())
    }

    async fn job_finished(&self, job_id: Uuid, success: bool) -> Result<(), Self::Error> {
        info!(job_id = %job_id, success = %success, "Job finished");
        let url = format!("{}/api/v1/agent/jobs/{}/finish", self.base_url, job_id);
        let request = JobFinishRequest { success };
        self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ReporterError(e.into()))?;
        Ok(())
    }
}
