use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rhai::{Dynamic, export_module};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::EnvDefinition;

#[derive(Debug)]
pub struct EvalContext {
    pub event: &'static str,
    pub branch: Option<String>,
    pub r#ref: String,
    pub tag: Option<String>,
    pub args: Dynamic,
    pub default_image: String,
    pub default_shell: String,

    pub pipelines: HashMap<Uuid, Rc<RefCell<Pipeline>>>,
}

#[derive(Debug)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub default_image: String,
    pub default_shell: String,

    pub jobs: HashMap<String, Rc<RefCell<Job>>>,
}

#[derive(Debug)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub env: HashMap<String, EnvDefinition>,
    pub shell: String,
    pub image: String,
    pub depends: Vec<String>,
    pub timeout: i32,

    pub steps: Vec<Rc<RefCell<Step>>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JobConfig {
    pub name: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, EnvDefinition>,
    pub shell: Option<String>,
    pub image: Option<String>,
    pub timeout: Option<i32>,
}

impl Job {
    fn new_from_config(
        key: String,
        config: JobConfig,
        default_image: &str,
        default_shell: &str,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: config.name.unwrap_or_else(|| key.clone()),
            key,
            env: config.env,
            shell: config.shell.unwrap_or_else(|| default_shell.into()),
            image: config.image.unwrap_or_else(|| default_image.into()),
            depends: Vec::new(),
            steps: Vec::new(),
            timeout: config.timeout.unwrap_or(30),
        }
    }
}

#[derive(Debug)]
pub struct Step {
    pub id: Uuid,
    pub name: String,
    pub command: String,
    pub env: HashMap<String, EnvDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepConfig {
    #[serde(default)]
    pub name: Option<String>,
    pub command: String,
    #[serde(default)]
    pub env: HashMap<String, EnvDefinition>,
}

#[export_module]
pub mod kanade_rhai_module {
    use std::{cell::RefCell, rc::Rc};

    use rhai::{
        EvalAltResult,
        serde::{from_dynamic, to_dynamic},
    };

    #[rhai_fn(get = "event")]
    pub fn eval_value(ctx: &mut Rc<RefCell<EvalContext>>) -> String {
        ctx.borrow().event.to_string()
    }

    #[rhai_fn(get = "branch")]
    pub fn eval_branch(ctx: &mut Rc<RefCell<EvalContext>>) -> String {
        ctx.borrow()
            .branch
            .as_ref()
            .map(|x| x.to_string())
            .unwrap_or_default()
    }

    #[rhai_fn(get = "tag")]
    pub fn eval_tag(ctx: &mut Rc<RefCell<EvalContext>>) -> String {
        ctx.borrow()
            .tag
            .as_ref()
            .map(|x| x.to_string())
            .unwrap_or_default()
    }

    #[rhai_fn(get = "args")]
    pub fn eval_args(ctx: &mut Rc<RefCell<EvalContext>>) -> Dynamic {
        ctx.borrow().args.clone()
    }

    #[rhai_fn(get = "ref")]
    pub fn eval_ref(ctx: &mut Rc<RefCell<EvalContext>>) -> String {
        ctx.borrow().r#ref.clone()
    }

    #[rhai_fn(name = "pipeline")]
    pub fn eval_pipeline(
        ctx: &mut Rc<RefCell<EvalContext>>,
        name: String,
    ) -> Rc<RefCell<Pipeline>> {
        let id = Uuid::now_v7();

        let pipeline = Rc::new(RefCell::new(Pipeline {
            id,
            name,
            jobs: HashMap::new(),
            default_image: ctx.borrow().default_image.clone(),
            default_shell: ctx.borrow().default_shell.clone(),
        }));

        ctx.borrow_mut().pipelines.insert(id, pipeline.clone());

        pipeline
    }

    #[rhai_fn(get = "id")]
    pub fn pipeline_id(pipeline: &mut Rc<RefCell<Pipeline>>) -> String {
        pipeline.borrow().id.to_string()
    }

    #[rhai_fn(get = "name")]
    pub fn pipeline_name(pipeline: &mut Rc<RefCell<Pipeline>>) -> String {
        pipeline.borrow().name.clone()
    }

    #[rhai_fn(name = "job")]
    pub fn pipeliine_job(pipeline: &mut Rc<RefCell<Pipeline>>, key: String) -> Rc<RefCell<Job>> {
        let job = Job::new_from_config(
            key.clone(),
            JobConfig::default(),
            &pipeline.borrow().default_image,
            &pipeline.borrow().default_shell,
        );
        let job = Rc::new(RefCell::new(job));

        pipeline.borrow_mut().jobs.insert(key, job.clone());

        job
    }

    #[rhai_fn(name = "job", return_raw)]
    pub fn pipeliine_job_with_config(
        pipeline: &mut Rc<RefCell<Pipeline>>,
        key: String,
        config: Dynamic,
    ) -> Result<Rc<RefCell<Job>>, Box<EvalAltResult>> {
        let job = Job::new_from_config(
            key.clone(),
            from_dynamic(&config)?,
            &pipeline.borrow().default_image,
            &pipeline.borrow().default_shell,
        );
        let job = Rc::new(RefCell::new(job));

        pipeline.borrow_mut().jobs.insert(key, job.clone());

        Ok(job)
    }

    #[rhai_fn(get = "id")]
    pub fn job_id(job: &mut Rc<RefCell<Job>>) -> String {
        job.borrow().id.to_string()
    }

    #[rhai_fn(get = "name")]
    pub fn job_name(job: &mut Rc<RefCell<Job>>) -> String {
        job.borrow().name.clone()
    }

    #[rhai_fn(get = "key")]
    pub fn job_key(job: &mut Rc<RefCell<Job>>) -> String {
        job.borrow().key.clone()
    }

    #[rhai_fn(get = "env", return_raw)]
    pub fn job_env(job: &mut Rc<RefCell<Job>>) -> Result<Dynamic, Box<EvalAltResult>> {
        to_dynamic(&job.borrow().env).map_err(Into::into)
    }

    #[rhai_fn(name = "depend")]
    pub fn job_depend(job: &mut Rc<RefCell<Job>>, key: String) -> Rc<RefCell<Job>> {
        job.borrow_mut().depends.push(key);
        job.clone()
    }

    #[rhai_fn(name = "step", return_raw)]
    pub fn job_step(
        job: &mut Rc<RefCell<Job>>,
        config: Dynamic,
    ) -> Result<Rc<RefCell<Step>>, Box<EvalAltResult>> {
        let config: StepConfig = from_dynamic(&config)?;

        let step = Rc::new(RefCell::new(Step {
            id: Uuid::now_v7(),
            name: config.name.unwrap_or_else(|| {
                config
                    .command
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .unwrap_or("Unnamed step")
                    .to_string()
            }),
            command: config.command,
            env: config.env,
        }));

        job.borrow_mut().steps.push(step.clone());

        Ok(step)
    }

    #[rhai_fn(get = "id")]
    pub fn step_id(step: &mut Rc<RefCell<Step>>) -> String {
        step.borrow().id.to_string()
    }

    #[rhai_fn(get = "name")]
    pub fn step_name(step: &mut Rc<RefCell<Step>>) -> String {
        step.borrow().name.clone()
    }
}
