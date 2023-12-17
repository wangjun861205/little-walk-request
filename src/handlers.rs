use actix_web::{
    error::{Error, ErrorInternalServerError, ErrorUnauthorized},
    web::{Data, Json, Path, Query},
    FromRequest, HttpRequest, HttpResponse, Result,
};
use futures::future::{ready, Ready};

use crate::core::{
    entities::WalkRequest,
    repository::{Pagination, Repository, WalkRequestCreate},
    service::Service,
};

use serde::{Deserialize, Serialize};

pub(crate) struct UserID(String);

impl FromRequest for UserID {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(user_id) = req.headers().get("X-User-ID") {
            match user_id.to_str() {
                Ok(user_id) => return ready(Ok(UserID(user_id.to_owned()))),
                Err(e) => return ready(Err(ErrorUnauthorized(e))),
            }
        }
        ready(Err(ErrorUnauthorized("无权限")))
    }
}

pub(crate) async fn create_walk_request<R>(
    service: Data<Service<R>>,
    UserID(user_id): UserID,
    Json(mut body): Json<WalkRequestCreate>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    body.created_by = user_id;
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

pub(crate) async fn my_walk_requests<R>(
    service: Data<Service<R>>,
    UserID(user_id): UserID,
    Query(pagination): Query<Pagination>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    let walk_requests = service
        .my_walk_requests(&user_id, Pagination::new(pagination.page, pagination.size))
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(walk_requests))
}

pub(crate) async fn accept<R>(
    service: Data<Service<R>>,
    path: Path<(String,)>,
    req: HttpRequest,
) -> Result<Json<WalkRequest>>
where
    R: Repository + Clone,
{
    let user_id = req
        .headers()
        .get("X-User-ID")
        .ok_or(ErrorUnauthorized("无权限"))?
        .to_str()
        .map_err(ErrorUnauthorized)?;
    service
        .accept(path.0.as_str(), user_id)
        .await
        .map_err(ErrorInternalServerError)
        .map(Json)
}

pub(crate) async fn remove_acceptance<R>(
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
    service
        .remove_acceptance(path.0.as_str(), user_id)
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn assign_accepter<R>(
    service: Data<Service<R>>,
    path: Path<(String, String)>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .assign_accepter(path.0.as_str(), path.1.as_str())
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn dismiss_accepter<R>(
    service: Data<Service<R>>,
    path: Path<(String, String)>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .dismiss_accepter(path.0.as_str(), path.1.as_str())
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn resign_acceptance<R>(
    service: Data<Service<R>>,
    path: Path<(String, String)>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .resign_acceptance(path.0.as_str(), path.1.as_str())
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn cancel_accepted_request<R>(
    service: Data<Service<R>>,
    path: Path<(String, String)>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .cancel_accepted_request(path.0.as_str(), path.1.as_str())
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn cancel_unaccepted_request<R>(
    service: Data<Service<R>>,
    path: Path<(String,)>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .cancel_unaccepted_request(path.0.as_str())
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn start_walk<R>(
    UserID(user_id): UserID,
    service: Data<Service<R>>,
    path: Path<(String,)>,
) -> Result<Json<WalkRequest>>
where
    R: Repository + Clone,
{
    service
        .start_walk(path.0.as_str(), &user_id)
        .await
        .map_err(ErrorInternalServerError)
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub(crate) struct Location {
    longitude: f64,
    latitude: f64,
}

pub(crate) async fn record_walking_location<R>(
    service: Data<Service<R>>,
    request_id: Path<(String,)>,
    Json(location): Json<Location>,
) -> Result<HttpResponse>
where
    R: Repository + Clone,
{
    service
        .record_walking_location(request_id.0.as_str(), location.longitude, location.latitude)
        .await
        .map_err(ErrorInternalServerError)
        .map(|_| HttpResponse::Ok().finish())
}

pub(crate) async fn finish_walk<R>(
    UserID(user_id): UserID,
    service: Data<Service<R>>,
    path: Path<(String,)>,
) -> Result<Json<WalkRequest>>
where
    R: Repository + Clone,
{
    service
        .finish_walk(path.0.as_str(), &user_id)
        .await
        .map_err(ErrorInternalServerError)
        .map(Json)
}
