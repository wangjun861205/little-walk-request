use chrono::{DateTime, Utc};
use little_walk_dog::core::entities::Dog;
use nb_field_names::FieldNames;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, FieldNames, Default)]
pub struct WalkRequest {
    pub id: String,
    pub dogs: Vec<Dog>,
    pub should_start_after: Option<DateTime<Utc>>,
    pub should_start_before: Option<DateTime<Utc>>,
    pub should_end_after: Option<DateTime<Utc>>,
    pub should_end_before: Option<DateTime<Utc>>,
    pub latitude: f64,
    pub longitude: f64,
    pub distance: Option<f64>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
    pub acceptances: Option<Vec<String>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, FieldNames, Default)]
pub struct WalkingLocation {
    pub id: String,
    pub request_id: String,
    pub longitude: f64,
    pub latitude: f64,
}
