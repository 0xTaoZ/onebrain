pub mod error;
pub mod lan;
pub mod middleware;
pub mod npx_browser_lifecycle;
pub mod parent_watch;
pub mod routes;

// #[cfg(feature = "cloud")]
// type DeploymentImpl = agent_chatgroup_cloud::deployment::CloudDeployment;
// #[cfg(not(feature = "cloud"))]
pub type DeploymentImpl = local_deployment::LocalDeployment;
