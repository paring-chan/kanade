use job_executor::adapter::{JobStatusReport, LogLine};
use uuid::Uuid;
use tracing::info;
use std::error::Error;

#[derive(Debug)]
pub struct ReporterError(pub anyhow::Error);

impl std::fmt::Display for ReporterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ReporterError {}

pub struct HttpReporter;

impl JobStatusReport for HttpReporter {
    type Error = ReporterError;

    async fn step_started(&self, step_id: Uuid, step_name: &str) -> Result<(), Self::Error> {
        info!(step_id = %step_id, step_name = %step_name, "Step started");
        Ok(())
    }

    async fn step_finished(&self, step_id: Uuid, exit_code: i32) -> Result<(), Self::Error> {
        info!(step_id = %step_id, exit_code = %exit_code, "Step finished");
        Ok(())
    }

    async fn step_log(&self, step_id: Uuid, line: LogLine) -> Result<(), Self::Error> {
        info!(step_id = %step_id, line = ?line, "Step log");
        Ok(())
    }

    async fn job_finished(&self, job_id: Uuid, success: bool) -> Result<(), Self::Error> {
        info!(job_id = %job_id, success = %success, "Job finished");
        Ok(())
    }
}
