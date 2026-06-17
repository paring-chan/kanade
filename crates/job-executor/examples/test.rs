use std::collections::HashMap;

use chrono::Duration;
use job_executor::{Job, JobExecutor, JobStep, adapter::LogLine};
use secrecy::SecretString;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
enum DummyError {
    #[error("dummy error")]
    Dummy,
}

#[derive(Debug, Clone)]
struct DummyReporter;

impl job_executor::adapter::JobStatusReport for DummyReporter {
    type Error = DummyError;

    async fn step_started(
        &self,
        _job_id: Uuid,
        _step_id: Uuid,
        step_name: &str,
    ) -> Result<(), Self::Error> {
        println!("Step started: {}", step_name);
        Ok(())
    }

    async fn step_finished(
        &self,
        _job_id: Uuid,
        _step_id: Uuid,
        exit_code: i32,
    ) -> Result<(), Self::Error> {
        println!("Step finished with code: {}", exit_code);
        Ok(())
    }

    async fn step_log(
        &self,
        _job_id: Uuid,
        _step_id: Uuid,
        line: LogLine,
    ) -> Result<(), Self::Error> {
        println!("Log: {:?}", line);
        Ok(())
    }

    async fn job_finished(&self, _job_id: Uuid, success: bool) -> Result<(), Self::Error> {
        println!("Job finished. Success: {}", success);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let job = Job {
        id: Uuid::new_v4(),
        image: "oven/bun:latest".to_string(),
        timeout: Duration::minutes(5),
        steps: vec![
            JobStep {
                id: Uuid::new_v4(),
                name: "Setup".to_string(),
                ordering: 0,
                command: r#"echo "console.log('Hello!!!!!')" > asdf.ts"#.to_string(),
                env: Default::default(),
            },
            JobStep {
                id: Uuid::new_v4(),
                name: "Test".to_string(),
                ordering: 1,
                command: "bun asdf.ts".to_string(),
                env: Default::default(),
            },
        ],
        env: HashMap::default(),
        secrets: HashMap::default(),
        ssh_key: SecretString::from("wowowowowowowo"),
    };

    let executor = JobExecutor::<DummyReporter>::new().unwrap();
    let reporter = DummyReporter;

    executor.run(job, &reporter).await.unwrap();
}
