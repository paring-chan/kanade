use std::collections::HashMap;

use poem_openapi::{payload::Json, ApiResponse, Object, Union};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct JobFinishRequest {
    pub success: bool,
}

#[derive(ApiResponse)]
pub enum JobFinishResponse {
    /// 작업 완료됨
    #[oai(status = 200)]
    Ok,
    /// 작업이 존재하지 않음
    ///
    /// 존재하지 않거나 이미 완료되었거나 agent 자신이 소유하지 않는 경우
    #[oai(status = 404)]
    JobNotFound,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct StepFinishRequest {
    pub success: bool,
    pub exit_code: i32,
}

#[derive(ApiResponse)]
pub enum StepFinishResponse {
    /// 스텝 완료됨
    #[oai(status = 200)]
    Ok,
    /// 스텝이 존재하지않음
    ///
    /// 1. 내 job의 스텝이 아님
    /// 2. 진짜 존재하지 않음
    /// 3. 이미 끝남
    #[oai(status = 404)]
    StepNotFound,
}

#[derive(ApiResponse)]
pub enum StepStartedResponse {
    /// 스텝 시작됨
    #[oai(status = 200)]
    Ok,
    /// 스텝이 존재하지않음
    ///
    /// 1. 내 job의 스텝이 아님
    /// 2. 진짜 존재하지 않음
    /// 3. 이미 끝남
    #[oai(status = 404)]
    StepNotFound,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct AgentPipelineJobStepResponse {
    /// 스텝 ID
    pub id: Uuid,
    /// 스텝 이름
    pub name: String,
    /// 스텝 순서(정렬용)
    pub ordering: i32,
    /// 실행 명령어
    pub command: String,
    /// 스텝 스코프 환경변수 목록
    pub env: HashMap<String, EnvDefinitionResponse>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct AgentPipelineJobResponse {
    /// 작업 ID
    pub id: Uuid,
    /// 작업 이름
    pub name: String,
    /// 작업 타임아웃 (분 단위)
    pub timeout: i32,
    /// 컨테이너 이미지
    pub image: Option<String>,
}

/// 환경변수 정의

#[derive(Union, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
#[oai(discriminator_name = "type", rename_all = "snake_case")]
pub enum EnvDefinitionResponse {
    Static(StaticEnv),
    Secret(SecretEnv),
}

/// 고정된 환경변수
#[derive(Object, Serialize, Deserialize, Debug)]
pub struct StaticEnv {
    /// 환경변수 값
    pub value: String,
}

/// 시크릿 환경변수
#[derive(Object, Serialize, Deserialize, Debug)]
pub struct SecretEnv {
    /// 환경변수를 불러올 시크릿 키
    pub secret_key: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct JobAcquireResponse {
    /// 작업 실행 ID
    pub id: Uuid,
    /// 작업 정보
    pub job: AgentPipelineJobResponse,
    /// 스텝 목록
    pub steps: Vec<AgentPipelineJobStepResponse>,
    /// Job 스코프 환경변수 목록
    pub env: HashMap<String, EnvDefinitionResponse>,
    /// 레퍼런스된 시크릿 목록
    pub secrets: HashMap<String, String>,
    pub ssh_key: String,
}

#[derive(ApiResponse)]
pub enum JobAcquireEndpointResponse {
    /// 작업 할당됨
    #[oai(status = 200)]
    Ok(Json<JobAcquireResponse>),

    /// 현재 할당 가능한 작업이 없음
    #[oai(status = 204)]
    NoContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentLogKind {
    Stdout,
    Stderr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum AgentLogMessage {
    Log {
        step_id: Uuid,
        kind: AgentLogKind,
        content: String,
    },
}
