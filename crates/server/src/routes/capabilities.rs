//! Public protocol metadata. Never include accounts, paths, credentials, or chat data.
use axum::{Json, Router, extract::State, routing::get};
use deployment::Deployment;
use executors::executors::CodingAgent;
use serde::Serialize;
use strum::VariantNames;
use ts_rs::TS;

use crate::DeploymentImpl;

#[derive(Debug, Serialize, TS)]
pub struct CapabilityLimits {
    pub message_length: u32,
    pub messages_per_hour: u32,
    pub members_per_session: u32,
    pub agent_chain_depth: u32,
}

#[derive(Debug, Serialize, TS)]
pub struct CapabilityLimitEnforcement {
    pub message_length: bool,
    pub messages_per_hour: bool,
    pub members_per_session: bool,
    pub agent_chain_depth: bool,
}

#[derive(Debug, Serialize, TS)]
pub struct CapabilityEndpoint {
    pub method: String,
    pub path: String,
    pub does: String,
}

#[derive(Debug, Serialize, TS)]
pub struct Capabilities {
    pub name: String,
    pub version: String,
    pub limits: CapabilityLimits,
    /// False means a recommended client budget, not a server rejection threshold.
    pub limit_enforcement: CapabilityLimitEnforcement,
    pub executors: Vec<String>,
    pub executor_scope: String,
    pub tools: Vec<String>,
    pub tool_scope: String,
    pub endpoints: Vec<CapabilityEndpoint>,
}

fn describe(chain_depth: u32) -> Capabilities {
    Capabilities {
        name: "openteams".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        limits: CapabilityLimits {
            message_length: 10_000,
            messages_per_hour: 100,
            members_per_session: 50,
            agent_chain_depth: chain_depth.max(1),
        },
        limit_enforcement: CapabilityLimitEnforcement {
            message_length: false,
            messages_per_hour: false,
            members_per_session: false,
            agent_chain_depth: true,
        },
        executors: CodingAgent::VARIANTS.iter().map(|name| name.to_ascii_lowercase()).collect(),
        executor_scope: "Supported adapters; query /api/agents/runtime for local availability.".into(),
        tools: ["read_file", "write_file", "run_tests", "git_diff"].map(str::to_owned).to_vec(),
        tool_scope: "Executor operations, subject to that member's permissions and tool configuration; these are not HTTP endpoints.".into(),
        endpoints: [
            ("GET", "/api/capabilities", "Read public capabilities and budget enforcement metadata"),
            ("GET", "/api/agents/runtime", "Discover installed executor availability"),
            ("GET", "/api/chat/sessions", "List local sessions"),
            ("POST", "/api/chat/sessions/{id}/messages", "Send with optional opKey; free-chat may return 409 needs_ack, confirm explicitly with force=true"),
            ("POST", "/api/chat/sessions/{id}/messages/upload", "Send attachments with the same opKey and acknowledgement protocol"),
            ("GET", "/api/chat/sessions/{id}/runtime", "Read delivery and member runtime state"),
        ].into_iter().map(|(method, path, does)| CapabilityEndpoint { method: method.into(), path: path.into(), does: does.into() }).collect(),
    }
}

async fn get_capabilities(State(deployment): State<DeploymentImpl>) -> Json<Capabilities> {
    let depth = deployment.config().read().await.max_agent_chain_depth;
    Json(describe(depth))
}

pub fn router() -> Router<DeploymentImpl> {
    Router::new().route("/capabilities", get(get_capabilities))
}

#[cfg(test)]
mod tests {
    #[test]
    fn metadata_matches_version_adapters_and_actual_enforcement() {
        let data = super::describe(12);
        assert_eq!(data.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(data.limits.agent_chain_depth, 12);
        assert_eq!(super::describe(0).limits.agent_chain_depth, 1);
        assert!(data.limit_enforcement.agent_chain_depth);
        assert!(!data.limit_enforcement.message_length);
        for name in ["claude_code", "codex", "deepseek_harness"] {
            assert!(data.executors.iter().any(|entry| entry == name));
        }
        let json = serde_json::to_value(data).unwrap();
        assert!(json.get("credentials").is_none());
        assert!(json.get("workspace_path").is_none());
    }
}
