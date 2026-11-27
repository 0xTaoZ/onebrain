use axum::{
    Json,
    extract::{Path, State},
    response::Json as ResponseJson,
};
use chrono::Utc;
use db::models::chat_schedule::{ChatSchedule, ChatScheduleInput};
use deployment::Deployment;
use serde::Serialize;
use services::services::scheduler;
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Serialize, TS)]
pub struct ChatSchedulePreview {
    /// Upcoming run times (RFC 3339, UTC).
    pub times: Vec<String>,
}

pub async fn list_schedules(
    State(deployment): State<DeploymentImpl>,
    Path(session_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<Vec<ChatSchedule>>>, ApiError> {
    let schedules = ChatSchedule::list_for_session(&deployment.db().pool, session_id).await?;
    Ok(ResponseJson(ApiResponse::success(schedules)))
}

pub async fn create_schedule(
    State(deployment): State<DeploymentImpl>,
    Path(session_id): Path<Uuid>,
    Json(input): Json<ChatScheduleInput>,
) -> Result<ResponseJson<ApiResponse<ChatSchedule>>, ApiError> {
    let next = scheduler::validate(&input, Utc::now()).map_err(ApiError::BadRequest)?;
    let schedule = ChatSchedule::create(&deployment.db().pool, session_id, &input, next).await?;
    scheduler::wake_scheduler();
    Ok(ResponseJson(ApiResponse::success(schedule)))
}

pub async fn update_schedule(
    State(deployment): State<DeploymentImpl>,
    Path(schedule_id): Path<Uuid>,
    Json(input): Json<ChatScheduleInput>,
) -> Result<ResponseJson<ApiResponse<ChatSchedule>>, ApiError> {
    let next = scheduler::validate(&input, Utc::now()).map_err(ApiError::BadRequest)?;
    let schedule = ChatSchedule::update(&deployment.db().pool, schedule_id, &input, next)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Schedule not found.".to_string()))?;
    scheduler::wake_scheduler();
    Ok(ResponseJson(ApiResponse::success(schedule)))
}

pub async fn delete_schedule(
    State(deployment): State<DeploymentImpl>,
    Path(schedule_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    ChatSchedule::delete(&deployment.db().pool, schedule_id).await?;
    scheduler::wake_scheduler();
    Ok(ResponseJson(ApiResponse::success(())))
}

pub async fn run_schedule_now(
    State(deployment): State<DeploymentImpl>,
    Path(schedule_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    if !ChatSchedule::run_now(&deployment.db().pool, schedule_id, Utc::now()).await? {
        return Err(ApiError::BadRequest("Schedule not found.".to_string()));
    }
    scheduler::wake_scheduler();
    Ok(ResponseJson(ApiResponse::success(())))
}

/// The next five run times of a schedule that is being edited.
pub async fn preview_schedule(
    Json(input): Json<ChatScheduleInput>,
) -> Result<ResponseJson<ApiResponse<ChatSchedulePreview>>, ApiError> {
    let now = Utc::now();
    scheduler::validate(&input, now).map_err(ApiError::BadRequest)?;
    let times = scheduler::preview(&input, now, 5)
        .into_iter()
        .map(|time| time.to_rfc3339())
        .collect();
    Ok(ResponseJson(ApiResponse::success(ChatSchedulePreview {
        times,
    })))
}
