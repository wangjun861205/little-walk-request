use crate::core::entities::WalkRequest;
use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalkRequestCreate {
    pub dog_ids: Vec<String>,
    pub should_start_after: DateTime<Utc>,
    pub should_start_before: DateTime<Utc>,
    pub should_end_before: DateTime<Utc>,
    pub should_end_after: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalkRequestUpdate {
    pub dog_ids: Option<Vec<String>>,
    pub should_start_after: Option<DateTime<Utc>>,
    pub should_start_before: Option<DateTime<Utc>>,
    pub should_end_before: Option<DateTime<Utc>>,
    pub should_end_after: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accepted_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WalkRequestQuery {
    pub dog_ids_includes_all: Option<Vec<String>>,
    pub dog_ids_includes_any: Option<Vec<String>>,
    pub nearby: Option<Vec<f64>>,
    pub accepted_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SortBy {
    pub field: String,
    pub order: Order,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub size: i64,
}

impl Pagination {
    pub fn new(page: i64, size: i64) -> Self {
        Self { page, size }
    }
}

pub trait Repository {
    async fn create_walk_request(&self, request: WalkRequestCreate) -> Result<String, Error>;
    async fn update_walk_request(&self, id: &str, request: WalkRequestUpdate)
        -> Result<u64, Error>;
    async fn get_walk_request(&self, request: WalkRequestQuery) -> Result<WalkRequest, Error>;
    async fn query_walk_requests(
        &self,
        query: WalkRequestQuery,
        sort_by: Option<SortBy>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<WalkRequest>, Error>;
}
