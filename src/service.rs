//! FGP service implementation for Neon.

use anyhow::Result;
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::NeonClient;

/// FGP service for Neon operations.
pub struct NeonService {
    client: Arc<NeonClient>,
    runtime: Runtime,
}

impl NeonService {
    /// Create a new NeonService with the given API key and org_id.
    pub fn new(api_key: String, org_id: String) -> Result<Self> {
        let client = NeonClient::new(api_key, org_id)?;
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
        let limit = Self::get_param_i32(&params, "limit", 10);
        let client = self.client.clone();

        let projects = self
            .runtime
            .block_on(async move { client.list_projects(Some(limit)).await })?;

        Ok(serde_json::json!({
            "projects": projects,
            "count": projects.len(),
        }))
    }

    /// Get project details implementation.
    fn get_project(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();

        let client = self.client.clone();

        let project = self
            .runtime
            .block_on(async move { client.get_project(&project_id).await })?;

        Ok(serde_json::to_value(project)?)
    }

    /// List branches implementation.
    fn list_branches(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();

        let client = self.client.clone();

        let branches = self
            .runtime
            .block_on(async move { client.list_branches(&project_id).await })?;

        Ok(serde_json::json!({
            "branches": branches,
            "count": branches.len(),
        }))
    }

    /// List databases implementation.
    fn list_databases(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: branch_id"))?
            .to_string();

        let client = self.client.clone();

        let databases = self
            .runtime
            .block_on(async move { client.list_databases(&project_id, &branch_id).await })?;

        Ok(serde_json::json!({
            "databases": databases,
            "count": databases.len(),
        }))
    }

    /// Get tables implementation.
    fn get_tables(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: branch_id"))?
            .to_string();
        let database = Self::get_param_str(&params, "database")
            .unwrap_or("neondb")
            .to_string();

        let client = self.client.clone();

        let tables = self
            .runtime
            .block_on(async move { client.get_tables(&project_id, &branch_id, &database).await })?;

        Ok(tables)
    }

    /// Get table schema implementation.
    fn get_table_schema(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: branch_id"))?
            .to_string();
        let database = Self::get_param_str(&params, "database")
            .unwrap_or("neondb")
            .to_string();
        let table = Self::get_param_str(&params, "table")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: table"))?
            .to_string();

        let client = self.client.clone();

        let schema = self.runtime.block_on(async move {
            client
                .get_table_schema(&project_id, &branch_id, &database, &table)
                .await
        })?;

        Ok(schema)
    }

    /// Run SQL query implementation.
    fn run_sql(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: branch_id"))?
            .to_string();
        let database = Self::get_param_str(&params, "database")
            .unwrap_or("neondb")
            .to_string();
        let query = Self::get_param_str(&params, "query")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?
            .to_string();

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client
                .run_sql(&project_id, &branch_id, &database, &query)
                .await
        })?;

        Ok(result)
    }

    /// Get user info implementation.
    fn get_user(&self) -> Result<Value> {
        let client = self.client.clone();

        let user = self
            .runtime
            .block_on(async move { client.get_user().await })?;

        Ok(user)
    }

    /// Create branch implementation.
    fn create_branch(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let name = Self::get_param_str(&params, "name").map(|s| s.to_string());
        let parent_id = Self::get_param_str(&params, "parent_id").map(|s| s.to_string());

        let client = self.client.clone();

        let branch = self.runtime.block_on(async move {
            client
                .create_branch(&project_id, name.as_deref(), parent_id.as_deref())
                .await
        })?;

        Ok(serde_json::to_value(branch)?)
    }

    /// Delete branch implementation.
    fn delete_branch(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: branch_id"))?
            .to_string();

        let client = self.client.clone();

        self.runtime
            .block_on(async move { client.delete_branch(&project_id, &branch_id).await })?;

        Ok(serde_json::json!({ "deleted": true }))
    }

    /// Get connection string implementation.
    fn get_connection_string(&self, params: HashMap<String, Value>) -> Result<Value> {
        let project_id = Self::get_param_str(&params, "project_id")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: project_id"))?
            .to_string();
        let branch_id = Self::get_param_str(&params, "branch_id").map(|s| s.to_string());
        let database = Self::get_param_str(&params, "database").map(|s| s.to_string());
        let pooled = params
            .get("pooled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client
                .get_connection_string(
                    &project_id,
                    branch_id.as_deref(),
                    database.as_deref(),
                    pooled,
                )
                .await
        })?;

        Ok(result)
    }
}

impl FgpService for NeonService {
    fn name(&self) -> &str {
        "neon"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "projects" | "neon.projects" => self.list_projects(params),
            "project" | "neon.project" => self.get_project(params),
            "branches" | "neon.branches" => self.list_branches(params),
            "databases" | "neon.databases" => self.list_databases(params),
            "tables" | "neon.tables" => self.get_tables(params),
            "schema" | "neon.schema" => self.get_table_schema(params),
            "sql" | "neon.sql" => self.run_sql(params),
            "user" | "neon.user" => self.get_user(),
            "create_branch" | "neon.create_branch" => self.create_branch(params),
            "delete_branch" | "neon.delete_branch" => self.delete_branch(params),
            "connection_string" | "neon.connection_string" => self.get_connection_string(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo {
                name: "neon.projects".into(),
                description: "List all Neon projects".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(serde_json::json!(10)),
                }],
            },
            MethodInfo {
                name: "neon.project".into(),
                description: "Get a specific project".into(),
                params: vec![ParamInfo {
                    name: "project_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "neon.branches".into(),
                description: "List branches for a project".into(),
                params: vec![ParamInfo {
                    name: "project_id".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "neon.databases".into(),
                description: "List databases for a branch".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "neon.tables".into(),
                description: "List tables in a database".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "database".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("neondb")),
                    },
                ],
            },
            MethodInfo {
                name: "neon.schema".into(),
                description: "Get table schema".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "database".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("neondb")),
                    },
                    ParamInfo {
                        name: "table".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "neon.sql".into(),
                description: "Run a SQL query".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "database".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("neondb")),
                    },
                    ParamInfo {
                        name: "query".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "neon.user".into(),
                description: "Get current user info".into(),
                params: vec![],
            },
            MethodInfo {
                name: "neon.create_branch".into(),
                description: "Create a new branch".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "name".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "parent_id".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "neon.delete_branch".into(),
                description: "Delete a branch".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "neon.connection_string".into(),
                description: "Get connection string for a branch".into(),
                params: vec![
                    ParamInfo {
                        name: "project_id".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "branch_id".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "database".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("neondb")),
                    },
                    ParamInfo {
                        name: "pooled".into(),
                        param_type: "boolean".into(),
                        required: false,
                        default: Some(serde_json::json!(false)),
                    },
                ],
            },
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("NeonService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Neon API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Neon API returned unsuccessful response");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Neon API: {}", e);
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
                checks.insert(
                    "neon_api".into(),
                    HealthStatus::healthy_with_latency(latency),
                );
            }
            Ok(false) => {
                checks.insert(
                    "neon_api".into(),
                    HealthStatus::unhealthy("API returned error"),
                );
            }
            Err(e) => {
                checks.insert("neon_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
