use axum::{
    Json,
    extract::{Path, Query, State},
    response::Json as ResponseJson,
};
use db::models::{
    brain_entry::{BrainEntry, BrainEntryKind, BrainEntryStatus, NewBrainEntry},
    chat_session::ChatSession,
};
use deployment::Deployment;
use serde::Deserialize;
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

#[derive(Debug, Deserialize)]
pub struct BrainListQuery {
    pub status: Option<BrainEntryStatus>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateBrainEntry {
    pub kind: BrainEntryKind,
    pub title: String,
    pub body: String,
    /// Share with every group instead of only this project folder.
    #[serde(default)]
    pub everywhere: bool,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateBrainEntry {
    pub kind: Option<BrainEntryKind>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<BrainEntryStatus>,
    /// Approve this proposal as the replacement of an existing entry.
    pub supersedes_id: Option<Uuid>,
}

async fn scope_of(deployment: &DeploymentImpl, session_id: Uuid) -> Result<String, ApiError> {
    Ok(ChatSession::find_by_id(&deployment.db().pool, session_id)
        .await?
        .and_then(|session| session.default_workspace_path)
        .unwrap_or_default())
}

/// Entries visible to this group: its project folder's and the shared ones.
pub async fn list_brain_entries(
    State(deployment): State<DeploymentImpl>,
    Path(session_id): Path<Uuid>,
    Query(query): Query<BrainListQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<BrainEntry>>>, ApiError> {
    let scope = scope_of(&deployment, session_id).await?;
    let entries = BrainEntry::list(
        &deployment.db().pool,
        &scope,
        query.status.unwrap_or(BrainEntryStatus::Active),
    )
    .await?;
    Ok(ResponseJson(ApiResponse::success(entries)))
}

/// An entry the user writes is active straight away and locked.
pub async fn create_brain_entry(
    State(deployment): State<DeploymentImpl>,
    Path(session_id): Path<Uuid>,
    Json(input): Json<CreateBrainEntry>,
) -> Result<ResponseJson<ApiResponse<BrainEntry>>, ApiError> {
    if input.title.trim().is_empty() || input.body.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Write a title and the content.".to_string(),
        ));
    }
    let scope = if input.everywhere {
        String::new()
    } else {
        scope_of(&deployment, session_id).await?
    };
    let pool = &deployment.db().pool;
    let entry = BrainEntry::create(
        pool,
        &NewBrainEntry {
            scope,
            kind: input.kind,
            title: input.title.trim().to_string(),
            body: input.body.trim().to_string(),
            status: BrainEntryStatus::Active,
            source_session_id: Some(session_id),
            source_run_id: None,
            source_agent: None,
        },
    )
    .await?
    .ok_or_else(|| ApiError::Conflict("This is already in the shared brain.".to_string()))?;
    let entry = BrainEntry::update(pool, entry.id, None, None, None, None, true)
        .await?
        .unwrap_or(entry);
    Ok(ResponseJson(ApiResponse::success(entry)))
}

pub async fn update_brain_entry(
    State(deployment): State<DeploymentImpl>,
    Path(entry_id): Path<Uuid>,
    Json(input): Json<UpdateBrainEntry>,
) -> Result<ResponseJson<ApiResponse<BrainEntry>>, ApiError> {
    let pool = &deployment.db().pool;
    let edited = input.title.is_some() || input.body.is_some();
    let entry = BrainEntry::update(
        pool,
        entry_id,
        input.kind,
        input.title.as_deref().map(str::trim),
        input.body.as_deref().map(str::trim),
        input.status,
        edited,
    )
    .await?
    .ok_or_else(|| ApiError::BadRequest("Entry not found.".to_string()))?;
    if let Some(old) = input.supersedes_id {
        BrainEntry::supersede(pool, entry_id, old).await?;
    }
    let entry = BrainEntry::find(pool, entry_id).await?.unwrap_or(entry);
    Ok(ResponseJson(ApiResponse::success(entry)))
}

pub async fn delete_brain_entry(
    State(deployment): State<DeploymentImpl>,
    Path(entry_id): Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    BrainEntry::delete(&deployment.db().pool, entry_id).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}
