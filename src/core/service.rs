use super::{
    entities::WalkRequest,
    repository::{Pagination, Repository, WalkRequestCreate, WalkRequestQuery},
};
use anyhow::Error;

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
}
