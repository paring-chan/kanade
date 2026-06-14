use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use rhai::{Engine, Scope, exported_module};

use crate::script::EvalContext;
use crate::types::{EvaluatedJob, EvaluatedPipeline, EvaluatedStep};

pub mod script;
pub mod types;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing dependency in pipeline {pipeline}: {job} -> {upstream}")]
    MissingDependency {
        pipeline: String,
        job: String,
        upstream: String,
    },

    #[error("job cycle detected in pipeline {pipeline}: {jobs:?}")]
    CycleDetected { pipeline: String, jobs: Vec<String> },

    #[error("rhai error: {0}")]
    Rhai(#[from] Box<rhai::EvalAltResult>),
}

pub fn evaluate(script: &str, context: EvalContext) -> crate::Result<Vec<EvaluatedPipeline>> {
    let mut engine = Engine::new();
    let module = exported_module!(script::kanade_rhai_module);
    engine.register_global_module(module.into());

    let ctx = Rc::new(RefCell::new(context));

    let mut scope = Scope::new();

    scope.push("ctx", ctx.clone());

    engine.run_with_scope(&mut scope, script)?;

    let mut pipelines = Vec::new();

    let ctx = ctx.borrow();
    for p in ctx.pipelines.values() {
        let p = p.borrow();

        let jobs = p
            .jobs
            .iter()
            .filter(|j| !j.1.borrow().steps.is_empty())
            .collect::<HashMap<_, _>>();

        if jobs.is_empty() {
            continue;
        }

        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();

        for key in jobs.keys() {
            let idx = graph.add_node(key.to_string());
            node_indices.insert(*key, idx);
        }

        for (key, job_cell) in &jobs {
            let job = job_cell.borrow();
            let current_idx = node_indices[key];

            for upstream_key in &job.depends {
                let upstream_idx =
                    node_indices
                        .get(upstream_key)
                        .ok_or_else(|| Error::MissingDependency {
                            pipeline: p.name.clone(),
                            job: key.to_string(),
                            upstream: upstream_key.clone(),
                        })?;

                graph.add_edge(*upstream_idx, current_idx, ());
            }
        }

        let sorted_indices = toposort(&graph, None).map_err(|cycle| {
            let failed_node_key = graph[cycle.node_id()].clone();
            Error::CycleDetected {
                pipeline: p.name.clone(),
                jobs: vec![failed_node_key],
            }
        })?;

        let mut evaluated_jobs = Vec::new();

        for index in sorted_indices {
            let k = &graph[index];
            let job = jobs[k].borrow();

            let parent_ids = job
                .depends
                .iter()
                .filter_map(|d| jobs.get(d).map(|x| x.borrow().id))
                .collect::<Vec<_>>();

            let steps = job
                .steps
                .iter()
                .map(|s| {
                    let s = s.borrow();
                    EvaluatedStep {
                        id: s.id,
                        name: s.name.clone(),
                        command: s.command.clone(),
                        env: s.env.clone(),
                    }
                })
                .collect::<Vec<_>>();

            let evaluated_job = EvaluatedJob {
                id: job.id,
                name: job.name.clone(),
                key: job.key.clone(),
                env: job.env.clone(),
                shell: job.shell.clone(),
                image: job.image.clone(),
                parents: parent_ids,
                steps,
            };

            evaluated_jobs.push(evaluated_job);
        }

        pipelines.push(EvaluatedPipeline {
            id: p.id,
            name: p.name.clone(),
            jobs: evaluated_jobs,
        });
    }

    Ok(pipelines)
}
