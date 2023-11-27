use chrono::{DateTime, Utc};
use nb_field_names::FieldNames;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, FieldNames, Default)]
pub struct WalkRequest {
    pub id: String,
    pub dog_ids: Vec<String>,
    pub should_start_after: DateTime<Utc>,
    pub should_start_before: DateTime<Utc>,
    pub should_end_after: DateTime<Utc>,
    pub should_end_before: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
    pub distance: Option<f64>,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}