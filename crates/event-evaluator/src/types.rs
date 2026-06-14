use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug)]
pub struct EvaluatedPipeline {
    pub id: Uuid,
    pub name: String,
    pub jobs: Vec<EvaluatedJob>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum EnvDefinition {
    Static(String),
    Secret { secret: String },
}

#[derive(Debug)]
pub struct EvaluatedJob {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub env: HashMap<String, EnvDefinition>,
    pub shell: String,
    pub image: String,
    pub parents: Vec<Uuid>,
    pub timeout: i32,

    pub steps: Vec<EvaluatedStep>,
}

#[derive(Debug)]
pub struct EvaluatedStep {
    pub id: Uuid,
    pub name: String,
    pub command: String,
    pub env: HashMap<String, EnvDefinition>,
}
