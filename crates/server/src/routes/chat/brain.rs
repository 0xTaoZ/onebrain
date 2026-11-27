//! The team brain as files: overview with token cost, file contents,
//! per-file history from git, old versions, restore and search.

use axum::{Json, extract::Query, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};
use services::services::brain::{self, BrainChange, BrainOverview, BrainSearchHit};
use ts_rs::TS;
use utils::response::ApiResponse;

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct BrainPathQuery {
    pub path: String,
    pub commit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BrainSearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct BrainRestoreRequest {
    pub path: String,
    pub commit: String,
}

#[derive(Debug, Serialize, TS)]
pub struct BrainFileContent {
    pub path: String,
    pub text: String,
    pub history: Vec<BrainChange>,
}

fn bad(err: impl std::fmt::Display) -> ApiError {
    ApiError::BadRequest(err.to_string())
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, brain::BrainError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(bad)?
        .map_err(bad)
}

pub async fn get_brain() -> Result<ResponseJson<ApiResponse<BrainOverview>>, ApiError> {
    let overview = blocking(|| {
        let dir = brain::brain_dir();
        brain::ensure_brain(&dir)?;
        brain::overview(&dir)
    })
    .await?;
    Ok(ResponseJson(ApiResponse::success(overview)))
}

/// A file's text (or its text at `commit`) plus who changed it and when.
pub async fn get_brain_file(
    Query(query): Query<BrainPathQuery>,
) -> Result<ResponseJson<ApiResponse<BrainFileContent>>, ApiError> {
    let file = blocking(move || {
        let dir = brain::brain_dir();
        let text = match query.commit.as_deref() {
            Some(commit) => brain::read_at(&dir, &query.path, commit)?,
            None => brain::read(&dir, &query.path)?,
        };
        let history = brain::history(&dir, &query.path, 50)?;
        Ok(BrainFileContent {
            path: query.path,
            text,
            history,
        })
    })
    .await?;
    Ok(ResponseJson(ApiResponse::success(file)))
}

pub async fn restore_brain_file(
    Json(request): Json<BrainRestoreRequest>,
) -> Result<ResponseJson<ApiResponse<()>>, ApiError> {
    blocking(move || brain::restore(&brain::brain_dir(), &request.path, &request.commit)).await?;
    Ok(ResponseJson(ApiResponse::success(())))
}

pub async fn search_brain(
    Query(query): Query<BrainSearchQuery>,
) -> Result<ResponseJson<ApiResponse<Vec<BrainSearchHit>>>, ApiError> {
    let hits = blocking(move || Ok(brain::search(&brain::brain_dir(), &query.q, 100))).await?;
    Ok(ResponseJson(ApiResponse::success(hits)))
}
