//! Neon HTTP API client with connection pooling.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::models::{Branch, Database, Project};

const API_BASE: &str = "https://console.neon.tech/api/v2";

/// Neon HTTP API client with persistent connection.
pub struct NeonClient {
    client: Client,
    api_key: String,
    org_id: String,
}

impl NeonClient {
    /// Create a new Neon client with API key and org_id.
    pub fn new(api_key: String, org_id: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            api_key,
            org_id,
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", API_BASE, endpoint);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

    /// Check if the client can connect to Neon API.
    pub async fn ping(&self) -> Result<bool> {
        // Try to list projects (limited to 1) as a health check
        let url = format!("{}/projects?org_id={}&limit=1", API_BASE, self.org_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to ping Neon API")?;

        Ok(response.status().is_success())
    }

    /// List all projects.
    pub async fn list_projects(&self, limit: Option<i32>) -> Result<Vec<Project>> {
        let limit = limit.unwrap_or(10);
        let endpoint = format!("/projects?org_id={}&limit={}", self.org_id, limit);

        #[derive(Deserialize)]
        struct ProjectsResponse {
            projects: Vec<Project>,
        }

        let response: ProjectsResponse = self.get(&endpoint).await?;
        Ok(response.projects)
    }

    /// Get a specific project.
    pub async fn get_project(&self, project_id: &str) -> Result<Project> {
        let endpoint = format!("/projects/{}", project_id);

        #[derive(Deserialize)]
        struct ProjectResponse {
            project: Project,
        }

        let response: ProjectResponse = self.get(&endpoint).await?;
        Ok(response.project)
    }

    /// List branches for a project.
    pub async fn list_branches(&self, project_id: &str) -> Result<Vec<Branch>> {
        let endpoint = format!("/projects/{}/branches", project_id);

        #[derive(Deserialize)]
        struct BranchesResponse {
            branches: Vec<Branch>,
        }

        let response: BranchesResponse = self.get(&endpoint).await?;
        Ok(response.branches)
    }

    /// List databases for a project branch.
    pub async fn list_databases(&self, project_id: &str, branch_id: &str) -> Result<Vec<Database>> {
        let endpoint = format!("/projects/{}/branches/{}/databases", project_id, branch_id);

        #[derive(Deserialize)]
        struct DatabasesResponse {
            databases: Vec<Database>,
        }

        let response: DatabasesResponse = self.get(&endpoint).await?;
        Ok(response.databases)
    }

    /// Get database tables.
    pub async fn get_tables(&self, project_id: &str, branch_id: &str, database: &str) -> Result<Value> {
        // Use the SQL endpoint to query tables
        let query = "SELECT schemaname as schema, tablename as name FROM pg_catalog.pg_tables WHERE schemaname NOT IN ('pg_catalog', 'information_schema') ORDER BY schemaname, tablename";
        self.run_sql(project_id, branch_id, database, query).await
    }

    /// Get table schema.
    pub async fn get_table_schema(&self, project_id: &str, branch_id: &str, database: &str, table: &str) -> Result<Value> {
        let query = format!(
            "SELECT column_name, data_type, is_nullable::boolean, column_default FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
            table.replace('\'', "''") // Basic SQL injection prevention
        );
        self.run_sql(project_id, branch_id, database, &query).await
    }

    /// Run a SQL query via the Neon SQL endpoint.
    pub async fn run_sql(&self, project_id: &str, branch_id: &str, database: &str, query: &str) -> Result<Value> {
        // First, get the connection string / endpoint for this branch
        let endpoints_url = format!("{}/projects/{}/endpoints", API_BASE, project_id);

        #[derive(Deserialize)]
        struct EndpointsResponse {
            endpoints: Vec<Endpoint>,
        }

        #[derive(Deserialize)]
        struct Endpoint {
            id: String,
            host: String,
            branch_id: String,
        }

        let endpoints: EndpointsResponse = self
            .client
            .get(&endpoints_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .send()
            .await?
            .json()
            .await?;

        // Find the endpoint for this branch
        let endpoint = endpoints
            .endpoints
            .iter()
            .find(|e| e.branch_id == branch_id)
            .ok_or_else(|| anyhow::anyhow!("No endpoint found for branch {}", branch_id))?;

        // Execute SQL via the serverless driver endpoint
        // Neon's SQL API: POST https://{host}/sql
        let sql_url = format!("https://{}/sql", endpoint.host);

        let body = serde_json::json!({
            "query": query,
            "params": []
        });

        let response = self
            .client
            .post(&sql_url)
            .header("Neon-Connection-String", format!("postgres://{}:{}@{}/{}",
                "neondb_owner", // Default role
                self.api_key,
                endpoint.host,
                database
            ))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to execute SQL")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("SQL execution failed: {} - {}", status, text);
        }

        response.json().await.context("Failed to parse SQL response")
    }

    /// Get current user/account info.
    pub async fn get_user(&self) -> Result<Value> {
        self.get("/users/me").await
    }
}
