use std::io::ErrorKind;
use std::marker::PhantomData;
use std::{collections::HashMap, path::Path};

use api_types::{EnvDefinitionResponse, SecretEnv, StaticEnv};
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
use secrecy::{ExposeSecret, SecretString};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncWriteExt,
};
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
    pub env: HashMap<String, EnvDefinitionResponse>,
    pub secrets: HashMap<String, SecretString>,
    pub ssh_key: SecretString,
}

pub struct JobStep {
    pub id: Uuid,
    pub name: String,
    pub ordering: i32,
    pub command: String,
    pub env: HashMap<String, EnvDefinitionResponse>,
}

pub struct JobExecutor<R: adapter::JobStatusReport> {
    docker: Docker,
    _marker: PhantomData<R>,
}

pub struct JobResult {}

impl<R: adapter::JobStatusReport> JobExecutor<R> {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_defaults()?,
            _marker: PhantomData,
        })
    }

    pub async fn run(&self, job: Job, reporter: &R) -> crate::Result<JobResult> {
        if let Err(_) = self.docker.inspect_image(&job.image).await {
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
        }

        let dir = TempDir::new()?;
        let ssh_dir = dir.path().join(".ssh");
        create_dir_all(&ssh_dir).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).await?;
            tokio::fs::set_permissions(&ssh_dir, std::fs::Permissions::from_mode(0o755)).await?;
        }

        let mut key_file = tokio::fs::File::create(ssh_dir.join("id_ed25519")).await?;
        key_file
            .write_all(job.ssh_key.expose_secret().as_bytes())
            .await?;
        key_file.flush().await?;
        drop(key_file);

        let mut env = job
            .env
            .iter()
            .filter_map(|(k, v)| match v {
                EnvDefinitionResponse::Static(StaticEnv { value }) => Some(format!("{k}={value}")),
                EnvDefinitionResponse::Secret(SecretEnv { secret_key }) => job
                    .secrets
                    .get(secret_key)
                    .map(|v| format!("{k}={}", v.expose_secret())),
            })
            .collect::<Vec<_>>();

        env.extend_from_slice(&[
            format!("HOME=/workspace"),
            format!("GIT_SSH_COMMAND=ssh -i /workspace/.ssh/id_ed25519 -o UserKnownHostsFile=/workspace/.ssh/known_hosts -o StrictHostKeyChecking=accept-new")
        ]);

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
                    env: Some(env),
                    cmd: Some(vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "tail -f /dev/null".into(),
                    ]),
                    working_dir: Some("/workspace".to_string()),
                    host_config: Some(HostConfig {
                        auto_remove: Some(true),
                        binds: Some(vec![format!("{}:{}", dir.path().display(), "/workspace")]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        debug!("container created: {container:?}");

        let result = self
            .run_steps(
                job.id,
                &name,
                dir.path(),
                job.steps,
                job.ssh_key.expose_secret(),
                job.secrets,
                reporter,
            )
            .await;

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
            Ok(status) => {
                let _ = reporter.job_finished(job.id, status == 0).await;
                Ok(JobResult {})
            }
            Err(e) => {
                let _ = reporter.job_finished(job.id, false).await;
                Err(e)
            }
        }
    }

    #[instrument(skip(self, job_id, steps, reporter))]
    async fn run_steps(
        &self,
        job_id: Uuid,
        container_name: &str,
        work_dir: &Path,
        mut steps: Vec<JobStep>,
        ssh_key: &str,
        secrets: HashMap<String, SecretString>,
        reporter: &R,
    ) -> Result<i32> {
        self.docker
            .start_container(
                container_name,
                Some(StartContainerOptions {
                    ..Default::default()
                }),
            )
            .await?;

        debug!("container started: {container_name}");

        steps.sort_by(|a, b| a.ordering.cmp(&b.ordering));

        let cwd_path = work_dir.join("cwd");

        let mask_log = |input: &str| -> String {
            let mut result = if ssh_key.len() >= 6 {
                input.replace(ssh_key, "***")
            } else {
                input.to_string()
            };

            for secret in secrets
                .values()
                .map(|x| x.expose_secret())
                .filter(|x| x.len() >= 6)
            {
                result = result.replace(secret, "***");
            }

            result
        };

        for step in steps {
            reporter
                .step_started(job_id, step.id, &step.name)
                .await
                .map_err(|e| Error::Reporter(Box::new(e)))?;

            let cwd = match File::open(&cwd_path).await {
                Ok(mut file) => {
                    let mut string = String::new();
                    file.read_to_string(&mut string).await?;
                    string.trim().to_string()
                }
                Err(e) if e.kind() == ErrorKind::NotFound => "/workspace".to_string(),
                Err(e) => return Err(Error::Io(e)),
            };

            let env = step
                .env
                .iter()
                .filter_map(|(k, v)| match v {
                    EnvDefinitionResponse::Static(StaticEnv { value }) => {
                        Some(format!("{k}={value}"))
                    }
                    EnvDefinitionResponse::Secret(SecretEnv { secret_key }) => secrets
                        .get(secret_key)
                        .map(|v| format!("{k}={}", v.expose_secret())),
                })
                .collect::<Vec<_>>();

            let exec = self
                .docker
                .create_exec(
                    container_name,
                    CreateExecOptions::<String> {
                        working_dir: Some(cwd),
                        cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), step.command]),
                        attach_stderr: Some(true),
                        attach_stdout: Some(true),
                        env: Some(env),
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
                let raw_message = match &output {
                    LogOutput::StdErr { message } => Some((true, message)),
                    LogOutput::StdOut { message } => Some((false, message)),
                    LogOutput::Console { message } => Some((false, message)),
                    _ => None,
                };

                if let Some((is_stderr, message_bytes)) = raw_message {
                    let lossy_string = String::from_utf8_lossy(message_bytes.as_ref());
                    let masked_string = mask_log(&lossy_string);

                    let log_line = if is_stderr {
                        LogLine::StdErr(masked_string)
                    } else {
                        LogLine::StdOut(masked_string)
                    };

                    reporter
                        .step_log(job_id, step.id, log_line)
                        .await
                        .map_err(|e| Error::Reporter(Box::new(e)))?;
                }
            }

            let ExecInspectResponse { exit_code, .. } = self.docker.inspect_exec(&exec.id).await?;
            let exit_code = exit_code.unwrap_or_default() as i32;
            reporter
                .step_finished(job_id, step.id, exit_code)
                .await
                .map_err(|e| Error::Reporter(Box::new(e)))?;

            if exit_code != 0 {
                return Ok(exit_code);
            }
        }

        Ok(0)
    }
}
