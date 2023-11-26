use actix_web::{
    error::ErrorInternalServerError,
    web::{Data, Json, Query},
    HttpResponse, Result,
};

use crate::core::{
    repository::{Pagination, Repository, WalkRequestCreate},
    service::Service,
};

use serde::{Deserialize, Serialize};

pub(crate) async fn create_walk_request<R>(
    service: Data<Service<R>>,
    Json(body): Json<WalkRequestCreate>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .create_walk_request(body)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NearbyWalkRequestsParams {
    pub lat: f64,
    pub lon: f64,
    pub radius: f64,
    pub page: i64,
    pub size: i64,
}

pub(crate) async fn nearby_walk_requests<R>(
    service: Data<Service<R>>,
    Query(params): Query<NearbyWalkRequestsParams>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    let walk_requests = service
        .nearby_walk_requests(
            params.lat,
            params.lon,
            params.radius,
            Pagination::new(params.page, params.size),
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(walk_requests))
}
