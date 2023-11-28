use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorTooManyRequests, ErrorUnauthorized},
    web::{Data, Json, Path, Query},
    HttpRequest, HttpResponse, Result,
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
    pub latitude: f64,
    pub longitude: f64,
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
            params.latitude,
            params.longitude,
            params.radius,
            Pagination::new(params.page, params.size),
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(walk_requests))
}

pub(crate) async fn accept_request<R>(
    service: Data<Service<R>>,
    path: Path<(String,)>,
    req: HttpRequest,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    let user_id = req
        .headers()
        .get("X-User-ID")
        .ok_or(ErrorUnauthorized("无权限"))?
        .to_str()
        .map_err(ErrorUnauthorized)?;
    if service
        .accept_request(path.0.clone().as_str(), user_id)
        .await
        .map_err(ErrorInternalServerError)?
        == 0
    {
        return Err(ErrorBadRequest("请求不存在或已接受该请求"));
    }
    Ok(HttpResponse::Ok().finish())
}
