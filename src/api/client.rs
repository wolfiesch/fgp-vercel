//! Vercel REST API client with connection pooling.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::models::{Deployment, DeploymentEvent, Project, User};

const API_BASE: &str = "https://api.vercel.com";

/// Vercel REST API client with persistent connection.
pub struct VercelClient {
    client: Client,
    token: String,
}

impl VercelClient {
    /// Create a new Vercel client with access token.
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, token })
    }

    /// Make an authenticated GET request.
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, endpoint);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// Check if the client can connect to Vercel API.
    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/v2/user", API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to ping Vercel API")?;

        Ok(response.status().is_success())
    }

    /// List all projects.
    pub async fn list_projects(&self, limit: Option<i32>) -> Result<Vec<Project>> {
        let limit = limit.unwrap_or(20);
        let endpoint = format!("/v9/projects?limit={}", limit);

        #[derive(Deserialize)]
        struct ProjectsResponse {
            projects: Vec<Project>,
        }

        let response: ProjectsResponse = self.get(&endpoint).await?;
        Ok(response.projects)
    }

    /// Get a specific project by ID or name.
    pub async fn get_project(&self, project_id: &str) -> Result<Project> {
        let endpoint = format!("/v9/projects/{}", project_id);
        self.get(&endpoint).await
    }

    /// List deployments (optionally filtered by project).
    pub async fn list_deployments(
        &self,
        project_id: Option<&str>,
        limit: Option<i32>,
    ) -> Result<Vec<Deployment>> {
        let limit = limit.unwrap_or(20);
        let mut endpoint = format!("/v6/deployments?limit={}", limit);

        if let Some(pid) = project_id {
            endpoint.push_str(&format!("&projectId={}", pid));
        }

        #[derive(Deserialize)]
        struct DeploymentsResponse {
            deployments: Vec<Deployment>,
        }

        let response: DeploymentsResponse = self.get(&endpoint).await?;
        Ok(response.deployments)
    }

    /// Get a specific deployment by ID or URL.
    pub async fn get_deployment(&self, deployment_id: &str) -> Result<Deployment> {
        let endpoint = format!("/v13/deployments/{}", deployment_id);
        self.get(&endpoint).await
    }

    /// Get deployment events/logs.
    pub async fn get_deployment_events(&self, deployment_id: &str) -> Result<Vec<DeploymentEvent>> {
        let endpoint = format!("/v2/deployments/{}/events", deployment_id);
        self.get(&endpoint).await
    }

    /// Get current user info.
    pub async fn get_user(&self) -> Result<User> {
        #[derive(Deserialize)]
        struct UserResponse {
            user: User,
        }

        let response: UserResponse = self.get("/v2/user").await?;
        Ok(response.user)
    }

    /// Get user info as raw JSON.
    pub async fn get_user_raw(&self) -> Result<Value> {
        self.get("/v2/user").await
    }
}
