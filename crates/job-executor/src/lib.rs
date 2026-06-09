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

pub mod adapter;
use adapter::LogLine;

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

pub struct JobExecutor<R: adapter::JobStatusReport> {
    docker: Docker,
    reporter: R,
}

pub struct JobResult {}

impl<R: adapter::JobStatusReport> JobExecutor<R> {
    pub fn new(reporter: R) -> crate::Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_defaults()?,
            reporter,
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

        match result {
            Ok(_) => {
                let _ = self.reporter.job_finished(job.id, true).await;
                Ok(JobResult {})
            }
            Err(e) => {
                let _ = self.reporter.job_finished(job.id, false).await;
                Err(e)
            }
        }
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
            self.reporter.step_started(step.id, &step.name).await.map_err(|e| Error::Reporter(Box::new(e)))?;
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
                        self.reporter.step_log(
                            step.id,
                            LogLine::StdErr(String::from_utf8_lossy(message.as_ref()).to_string()),
                        ).await.map_err(|e| Error::Reporter(Box::new(e)))?;
                    }
                    LogOutput::StdOut { message } => {
                        self.reporter.step_log(
                            step.id,
                            LogLine::StdOut(String::from_utf8_lossy(message.as_ref()).to_string()),
                        ).await.map_err(|e| Error::Reporter(Box::new(e)))?;
                    }
                    LogOutput::Console { message } => {
                        self.reporter.step_log(
                            step.id,
                            LogLine::StdOut(String::from_utf8_lossy(message.as_ref()).to_string()),
                        ).await.map_err(|e| Error::Reporter(Box::new(e)))?;
                    }
                    _ => {}
                }
            }

            let ExecInspectResponse { exit_code, .. } = self.docker.inspect_exec(&exec.id).await?;
            let exit_code = exit_code.unwrap_or_default() as i32;
            self.reporter.step_finished(step.id, exit_code).await.map_err(|e| Error::Reporter(Box::new(e)))?;
        }

        Ok(())
    }
}
