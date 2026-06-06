use std::collections::HashMap;

use bollard::{
    Docker,
    container::LogOutput,
    exec::{CreateExecOptions, StartExecOptions, StartExecResults},
    plugin::{ContainerCreateBody, ExecInspectResponse, HostConfig},
    query_parameters::{
        CreateContainerOptions, CreateImageOptions, RemoveContainerOptions, StartContainerOptions,
    },
};
use chrono::Duration;
use futures_util::StreamExt;
use tracing::{debug, instrument};
use uuid::Uuid;

mod error;
pub use error::*;

pub struct Job {
    pub id: Uuid,
    pub image: String,
    pub timeout: Duration,
    pub steps: Vec<JobStep>,
}

pub struct JobStep {
    pub id: Uuid,
    pub name: String,
    pub ordering: i32,
    pub command: String,
}

pub struct JobExecutor {
    docker: Docker,
}

pub struct JobResult {}

impl JobExecutor {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_defaults()?,
        })
    }

    pub async fn run(&self, job: Job) -> crate::Result<JobResult> {
        let mut pull = self.docker.create_image(
            Some(CreateImageOptions {
                from_image: Some(job.image.clone()),
                ..Default::default()
            }),
            None,
            None,
        );

        while let Some(result) = pull.next().await {
            let status = result?;
            debug!("pull progress: {status:?}");
        }

        let name = format!("kanade-job--{}", job.id);
        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: Some(name.clone()),
                    ..Default::default()
                }),
                ContainerCreateBody {
                    image: Some(job.image),
                    labels: Some({
                        let mut map = HashMap::new();
                        map.insert(format!("kanade.mizuki.my/type"), "job".to_string());
                        map.insert(format!("kanade.mizuki.my/job-id"), job.id.to_string());

                        map
                    }),
                    stop_timeout: Some(job.timeout.num_seconds()),
                    cmd: Some(vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "mkdir /work && tail -f /dev/null".into(),
                    ]),
                    host_config: Some(HostConfig {
                        auto_remove: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        debug!("container created: {container:?}");

        let result = self.run_steps(&name, job.steps).await;

        self.docker
            .remove_container(
                &name,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        result?;

        Ok(JobResult {})
    }

    #[instrument(skip(self, steps))]
    async fn run_steps(&self, container_name: &str, steps: Vec<JobStep>) -> Result<()> {
        self.docker
            .start_container(
                container_name,
                Some(StartContainerOptions {
                    ..Default::default()
                }),
            )
            .await?;

        debug!("container started: {container_name}");

        for step in steps {
            let exec = self
                .docker
                .create_exec(
                    container_name,
                    CreateExecOptions::<String> {
                        working_dir: Some("/work".to_string()),
                        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), step.command]),
                        attach_stderr: Some(true),
                        attach_stdout: Some(true),
                        ..Default::default()
                    },
                )
                .await?;
            let StartExecResults::Attached { mut output, .. } = self
                .docker
                .start_exec(
                    &exec.id,
                    Some(StartExecOptions {
                        ..Default::default()
                    }),
                )
                .await?
            else {
                panic!("why detached");
            };

            while let Some(output) = output.next().await.transpose()? {
                match output {
                    LogOutput::StdErr { message } => {
                        debug!("[STDERR] {}", String::from_utf8_lossy(message.as_ref()))
                    }
                    LogOutput::StdOut { message } => {
                        debug!("[STDOUT] {}", String::from_utf8_lossy(message.as_ref()))
                    }
                    LogOutput::Console { message } => {
                        debug!("[CONSOLE] {}", String::from_utf8_lossy(message.as_ref()))
                    }
                    _ => {}
                }
            }

            let ExecInspectResponse { exit_code, .. } = self.docker.inspect_exec(&exec.id).await?;
            let exit_code = exit_code.unwrap_or_default();
            dbg!(exit_code);
        }

        Ok(())
    }
}
