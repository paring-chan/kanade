use serde::{Deserialize, Serialize};
use uuid::Uuid;
use poem_openapi::{ApiResponse, Object, payload::Json};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct PipelineJobStepResponse {
    /// 스텝 ID
    pub id: Uuid,
    /// 스텝 이름
    pub name: String,
    /// 스텝 순서(정렬용)
    pub ordering: i32,
    /// 실행 명령어
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct PipelineJobResponse {
    /// 작업 ID
    pub id: Uuid,
    /// 작업 이름
    pub name: String,
    /// 작업 타임아웃 (분 단위)
    pub timeout: i32,
    /// 컨테이너 이미지
    pub image: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct PipelineJobStepRunResponse {
    /// 스텝 실행 ID
    pub id: Uuid,
    /// 스텝 정보
    pub step: PipelineJobStepResponse,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct PipelineJobRunResponse {
    /// 작업 실행 ID
    pub id: Uuid,
    /// 작업 재시도 시리얼
    pub attempt_serial: i32,
    /// 작업 정보
    pub job: PipelineJobResponse,
    /// 스텝 목록
    pub steps: Vec<PipelineJobStepRunResponse>,
}

#[derive(ApiResponse)]
pub enum AcquireResponse {
    /// 작업 할당됨
    #[oai(status = 200)]
    Ok(Json<PipelineJobRunResponse>),

    /// 현재 할당 가능한 작업이 없음
    #[oai(status = 204)]
    NoContent,
}
