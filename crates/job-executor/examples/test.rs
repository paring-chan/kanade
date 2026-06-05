use chrono::Duration;
use job_executor::{Job, JobExecutor};
use tracing::level_filters::LevelFilter;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let job = Job {
        id: Uuid::now_v7(),
        image: "oven/bun:latest".to_string(),
        timeout: Duration::minutes(5),
        steps: vec![],
    };

    let executor = JobExecutor::new().unwrap();

    executor.run(job).await.unwrap();
}
