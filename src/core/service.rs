use std::default;

use super::{
    entities::WalkRequest,
    repository::{Pagination, Repository, WalkRequestCreate, WalkRequestQuery, WalkRequestUpdate},
};
use anyhow::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: Repository + Clone,
{
    repository: R,
}

impl<R> Service<R>
where
    R: Repository + Clone,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_walk_request(&self, request: WalkRequestCreate) -> Result<String, Error> {
        if request.should_start_after >= request.should_end_before {
            return Err(Error::msg("非法的开始时间范围"));
        }
        if request.should_end_after >= request.should_end_before {
            return Err(Error::msg("非法的结束时间范围"));
        }
        if request.should_start_after >= request.should_end_before {
            return Err(Error::msg("结束时间不得早于开始时间"));
        }
        self.repository.create_walk_request(request).await
    }

    pub async fn nearby_walk_requests(
        &self,
        latitute: f64,
        longitude: f64,
        radius: f64,
        pagination: Pagination,
    ) -> Result<Vec<WalkRequest>, Error> {
        self.repository
            .query_walk_requests(
                WalkRequestQuery {
                    nearby: Some(vec![longitude, latitute, radius]),
                    ..Default::default()
                },
                None,
                Some(pagination),
            )
            .await
    }

    pub async fn accept_request(&self, request_id: &str, user_id: &str) -> Result<u64, Error> {
        self.repository
            .update_walk_requests_by_query(
                WalkRequestQuery {
                    id: Some(request_id.to_owned()),
                    ..Default::default()
                },
                WalkRequestUpdate {
                    add_to_acceptances: Some(vec![user_id.to_owned()]),
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn assign_accepter(&self, request_id: &str, user_id: &str) -> Result<u64, Error> {
        self.repository
            .update_walk_requests_by_query(
                WalkRequestQuery {
                    id: Some(request_id.to_owned()),
                    acceptances_includes_all: Some(vec![user_id.to_owned()]),
                    ..Default::default()
                },
                WalkRequestUpdate {
                    accepted_by: Some(user_id.to_owned()),
                    accepted_at: Some(Utc::now()),
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn dismiss_aceepter(&self, request_id: &str) -> Result<u64, Error> {
        self.repository
            .update_walk_request(
                request_id,
                WalkRequestUpdate {
                    unset_accepted_by: true,
                    unset_accepted_at: true,
                    ..Default::default()
                },
            )
            .await
    }
}
