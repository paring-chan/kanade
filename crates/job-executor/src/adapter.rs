use std::future::Future;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum LogLine {
    StdIn(String),
    StdOut(String),
    StdErr(String),
}

pub trait JobStatusReport: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn step_started(
        &self,
        step_id: Uuid,
        step_name: &str,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + Sync;
    fn step_finished(
        &self,
        step_id: Uuid,
        exit_code: i32,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + Sync;
    fn step_log(
        &self,
        step_id: Uuid,
        line: LogLine,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + Sync;
    fn job_finished(
        &self,
        job_id: Uuid,
        success: bool,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + Sync;
}
