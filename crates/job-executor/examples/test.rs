use chrono::Duration;
use job_executor::{Job, JobExecutor, JobStep};
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
        steps: vec![
            JobStep {
                id: Uuid::now_v7(),
                name: "Setup".to_string(),
                ordering: 0,
                command: r#"echo "console.log('Hello!!!!!')" > asdf.ts"#.to_string(),
            },
            JobStep {
                id: Uuid::now_v7(),
                name: "Test".to_string(),
                ordering: 1,
                command: "bun asdf.ts".to_string(),
            },
        ],
    };

    let executor = JobExecutor::new().unwrap();

    executor.run(job).await.unwrap();
}
