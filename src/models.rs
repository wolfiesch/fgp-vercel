//! Data models for Vercel API responses.

use serde::{Deserialize, Serialize};

/// Vercel project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub node_version: Option<String>,
    #[serde(default)]
    pub latest_deployments: Option<Vec<DeploymentSummary>>,
}

/// Summary deployment info (included in project listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentSummary {
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub ready_state: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
}

/// Full Vercel deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    /// Deployment ID (can be "uid" or "id" depending on API version)
    #[serde(alias = "id")]
    pub uid: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub ready_state: String,
    #[serde(default)]
    pub state: Option<String>,
    /// Created timestamp in milliseconds
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub building_at: Option<i64>,
    #[serde(default)]
    pub ready: Option<i64>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub creator: Option<Creator>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

/// Deployment creator info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    #[serde(default)]
    pub uid: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

/// Deployment log event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Vercel user info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

/// Paginated response wrapper.
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    #[serde(alias = "projects", alias = "deployments")]
    pub items: Vec<T>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

/// Pagination info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub next: Option<i64>,
    #[serde(default)]
    pub prev: Option<i64>,
}
