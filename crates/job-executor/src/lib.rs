use std::collections::HashMap;

use bollard::{
    Docker,
    plugin::{ContainerCreateBody, HostConfig},
    query_parameters::{
        CreateContainerOptions, CreateImageOptions, RemoveContainerOptions, StartContainerOptions,
    },
};
use chrono::Duration;
use futures_util::StreamExt;
use tracing::debug;
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
                    cmd: Some(vec!["/bin/sh".into(), "-c".into(), "sleep infinity".into()]),
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

        Ok(())
    }
}
