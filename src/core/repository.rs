use crate::core::entities::WalkRequest;
use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::entities::Dog;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalkRequestCreate {
    pub dogs: Vec<Dog>,
    pub should_start_after: Option<DateTime<Utc>>,
    pub should_start_before: Option<DateTime<Utc>>,
    pub should_end_before: Option<DateTime<Utc>>,
    pub should_end_after: Option<DateTime<Utc>>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WalkRequestUpdate {
    pub dogs: Option<Vec<Dog>>,
    pub should_start_after: Option<DateTime<Utc>>,
    pub should_start_before: Option<DateTime<Utc>>,
    pub should_end_before: Option<DateTime<Utc>>,
    pub should_end_after: Option<DateTime<Utc>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub unset_accepted_by: bool,
    pub unset_accepted_at: bool,
    pub add_to_acceptances: Option<String>,
    pub remove_from_acceptances: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WalkRequestQuery {
    pub id: Option<String>,
    pub dog_ids_includes_all: Option<Vec<String>>,
    pub dog_ids_includes_any: Option<Vec<String>>,
    pub nearby: Option<Vec<f64>>,
    pub accepted_by: Option<String>,
    pub accepted_by_neq: Option<String>,
    pub accepted_by_is_null: bool,
    pub acceptances_includes_all: Option<Vec<String>>,
    pub acceptances_includes_any: Option<Vec<String>>,
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
    async fn update_walk_requests_by_query(
        &self,
        query: WalkRequestQuery,
        update: WalkRequestUpdate,
    ) -> Result<u64, Error>;
    async fn get_walk_request(&self, id: &str) -> Result<WalkRequest, Error>;
    async fn query_walk_requests(
        &self,
        query: WalkRequestQuery,
        sort_by: Option<SortBy>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<WalkRequest>, Error>;
}
