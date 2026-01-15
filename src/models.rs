//! Data models for Neon API responses.

use serde::{Deserialize, Serialize};

/// Neon project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub region_id: Option<String>,
    #[serde(default)]
    pub platform_id: Option<String>,
    #[serde(default)]
    pub pg_version: Option<i32>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Neon branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub current_state: Option<String>,
}

/// Neon database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub id: i64,
    pub branch_id: String,
    pub name: String,
    pub owner_name: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Database table info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    #[serde(default)]
    pub row_count: Option<i64>,
}

/// Table column info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    #[serde(default)]
    pub is_nullable: bool,
    #[serde(default)]
    pub column_default: Option<String>,
}

/// SQL query result.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub rows: Vec<Vec<serde_json::Value>>,
    #[serde(default)]
    pub row_count: i64,
}

/// Neon API list response wrapper.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ListResponse<T> {
    #[serde(alias = "projects", alias = "branches", alias = "databases")]
    pub items: Vec<T>,
}

/// Neon API error response.
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}
