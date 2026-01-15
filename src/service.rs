//! FGP service implementation for Vercel.

use anyhow::Result;
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::VercelClient;

/// FGP service for Vercel operations.
pub struct VercelService {
    client: Arc<VercelClient>,
    runtime: Runtime,
}

impl VercelService {
    /// Create a new VercelService with the given access token.
    pub fn new(token: String) -> Result<Self> {
        let client = VercelClient::new(token)?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    /// Helper to get a i32 parameter with default.
    fn get_param_i32(params: &HashMap<String, Value>, key: &str, default: i32) -> i32 {
        params
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default)
    }

    /// Helper to get a string parameter.
    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Health check implementation.
    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(serde_json::json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "api_connected": ok,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// List projects implementation.
    fn list_projects(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_i32(&params, "limit", 20);
        let client = self.client.clone();

        let projects = self.runtime.block_on(async move {
            client.list_projects(Some(limit)).await
        })?;

        Ok(serde_json::json!({
            "projects": projects,
            "count": projects.len(),
        }))
    }

    /// Get project details implementation.
    fn get_project(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .or_else(|| Self::get_param_str(&params, "name"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id or name"))?
            .to_string();

        let client = self.client.clone();

        let project = self.runtime.block_on(async move {
            client.get_project(&project_id).await
        })?;

        Ok(serde_json::to_value(project)?)
    }

    /// List deployments implementation.
    fn list_deployments(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id").map(|s| s.to_string());
        let limit = Self::get_param_i32(&params, "limit", 20);
        let client = self.client.clone();

        let deployments = self.runtime.block_on(async move {
            client.list_deployments(project_id.as_deref(), Some(limit)).await
        })?;

        Ok(serde_json::json!({
            "deployments": deployments,
            "count": deployments.len(),
        }))
    }

    /// Get single deployment implementation.
    fn get_deployment(&self, params: HashMap<String, Value>) -> Result<Value> {
        let deployment_id = Self::get_param_str(&params, "deployment_id")
            .or_else(|| Self::get_param_str(&params, "id"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: deployment_id"))?
            .to_string();

        let client = self.client.clone();

        let deployment = self.runtime.block_on(async move {
            client.get_deployment(&deployment_id).await
        })?;

        Ok(serde_json::to_value(deployment)?)
    }

    /// Get deployment logs/events implementation.
    fn get_deployment_logs(&self, params: HashMap<String, Value>) -> Result<Value> {
        let deployment_id = Self::get_param_str(&params, "deployment_id")
            .or_else(|| Self::get_param_str(&params, "id"))
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: deployment_id"))?
            .to_string();

        let client = self.client.clone();

        let events = self.runtime.block_on(async move {
            client.get_deployment_events(&deployment_id).await
        })?;

        Ok(serde_json::json!({
            "events": events,
            "count": events.len(),
        }))
    }

    /// Get user info implementation.
    fn get_user(&self) -> Result<Value> {
        let client = self.client.clone();

        let user = self.runtime.block_on(async move {
            client.get_user_raw().await
        })?;

        Ok(user)
    }

    /// List env vars implementation.
    fn list_env_vars(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let target = Self::get_param_str(&params, "target").map(|s| s.to_string());

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.list_env_vars(&project_id, target.as_deref()).await
        })?;

        Ok(result)
    }

    /// Set env var implementation.
    fn set_env_var(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let key = Self::get_param_str(&params, "key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?
            .to_string();
        let value = Self::get_param_str(&params, "value")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: value"))?
            .to_string();

        // Parse target array if provided
        let target: Option<Vec<String>> = params
            .get("target")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

        let env_type = Self::get_param_str(&params, "type").map(|s| s.to_string());

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            let target_refs: Option<Vec<&str>> = target.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
            client.set_env_var(&project_id, &key, &value, target_refs, env_type.as_deref()).await
        })?;

        Ok(result)
    }

    /// List domains implementation.
    fn list_domains(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.list_domains(&project_id).await
        })?;

        Ok(result)
    }

    /// Redeploy implementation.
    fn redeploy(&self, params: HashMap<String, Value>) -> Result<Value> {
        let deployment_id = Self::get_param_str(&params, "deployment_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: deployment_id"))?
            .to_string();

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.redeploy(&deployment_id).await
        })?;

        Ok(result)
    }
}

impl FgpService for VercelService {
    fn name(&self) -> &str {
        "vercel"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "projects" | "vercel.projects" => self.list_projects(params),
            "project" | "vercel.project" => self.get_project(params),
            "deployments" | "vercel.deployments" => self.list_deployments(params),
            "deployment" | "vercel.deployment" => self.get_deployment(params),
            "logs" | "vercel.logs" => self.get_deployment_logs(params),
            "user" | "vercel.user" => self.get_user(),
            "env_vars" | "vercel.env_vars" => self.list_env_vars(params),
            "set_env" | "vercel.set_env" => self.set_env_var(params),
            "domains" | "vercel.domains" => self.list_domains(params),
            "redeploy" | "vercel.redeploy" => self.redeploy(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo {
                name: "vercel.projects".into(),
                description: "List all Vercel projects".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(serde_json::json!(20)),
                }],
            },
            MethodInfo {
                name: "vercel.project".into(),
                description: "Get a specific project by ID or name".into(),
                params: vec![ParamInfo {
                    name: "project_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "vercel.deployments".into(),
                description: "List deployments (optionally filtered by project)".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "limit".into(),
                        param_type: "integer".into(),
                        required: false,
                        default: Some(serde_json::json!(20)),
                    },
                ],
            },
            MethodInfo {
                name: "vercel.deployment".into(),
                description: "Get a specific deployment by ID".into(),
                params: vec![ParamInfo {
                    name: "deployment_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "vercel.logs".into(),
                description: "Get deployment logs/events".into(),
                params: vec![ParamInfo {
                    name: "deployment_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "vercel.user".into(),
                description: "Get current user info".into(),
                params: vec![],
            },
            MethodInfo {
                name: "vercel.env_vars".into(),
                description: "List environment variables for a project".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "target".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "vercel.set_env".into(),
                description: "Set an environment variable".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "key".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "value".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "target".into(),
                        param_type: "array".into(),
                        required: false,
                        default: Some(serde_json::json!(["production", "preview", "development"])),
                    },
                    ParamInfo {
                        name: "type".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("encrypted")),
                    },
                ],
            },
            MethodInfo {
                name: "vercel.domains".into(),
                description: "List domains for a project".into(),
                params: vec![ParamInfo {
                    name: "project_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "vercel.redeploy".into(),
                description: "Redeploy a deployment".into(),
                params: vec![ParamInfo {
                    name: "deployment_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("VercelService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Vercel API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Vercel API returned unsuccessful response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Vercel API: {}", e);
                    Err(e)
                }
            }
        })
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let client = self.client.clone();
        let start = std::time::Instant::now();
        let result = self.runtime.block_on(async move { client.ping().await });

        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(true) => {
                checks.insert("vercel_api".into(), HealthStatus::healthy_with_latency(latency));
            }
            Ok(false) => {
                checks.insert("vercel_api".into(), HealthStatus::unhealthy("API returned error"));
            }
            Err(e) => {
                checks.insert("vercel_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
