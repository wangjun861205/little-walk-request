use mongodb::bson::{from_document, Document};
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Database,
};

use crate::core::entities::WalkRequest;
use crate::core::repository::{Order, Pagination, Repository, SortBy};
use crate::core::repository::{WalkRequestCreate, WalkRequestQuery, WalkRequestUpdate};
use anyhow::Error;
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;

lazy_static! {
    static ref WALK_REQUEST_PROJECTION: Document = doc! {
        "id": {"$toString": "$_id"},
        "dog_ids": "$dog_ids",
        "should_start_after": {"$dateToString": {"date":"$should_start_after", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "should_start_before": {"$dateToString": {"date":"$should_start_before", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "should_end_after": {"$dateToString": {"date":"$should_end_after", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "should_end_before": {"$dateToString": {"date":"$should_end_before", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "longitude": { "$arrayElemAt": [ "$location.coordinates", 0]},
        "latitude": { "$arrayElemAt": [ "$location.coordinates", 1]},
        "distance": "$distance",
        "accepted_by": "$accepted_by",
        "accepted_at": {"$dateToString": {"date":"$accepted_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "started_at": {"$dateToString": {"date":"$started_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "finished_at": {"$dateToString": {"date":"$finished_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "created_at": {"$dateToString": {"date":"$created_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        "updated_at": {"$dateToString": {"date":"$updated_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
    };
}

#[derive(Debug, Clone)]
pub struct Mongodb {
    db: Database,
}

impl Mongodb {
    pub fn new(db: Database) -> Self {
        Mongodb { db }
    }
}

impl Repository for Mongodb {
    async fn create_walk_request(&self, request: WalkRequestCreate) -> Result<String, Error> {
        let inserted = self
            .db
            .collection("walk_requests")
            .insert_one(
                doc! {
                    "dog_ids": request.dog_ids,
                    "should_start_after": request.should_start_after,
                    "should_start_before": request.should_start_before,
                    "should_end_before": request.should_end_before,
                    "should_end_after": request.should_end_after,
                    "location": { "type": "Point", "coordinates": [request.longitude, request.latitude] },
                    "created_at": Utc::now(),
                    "updated_at": Utc::now(),
                },
                None,
            )
            .await?;
        Ok(inserted.inserted_id.to_string())
    }

    async fn get_walk_request(&self, id: &str) -> Result<WalkRequest, Error> {
        self.db
            .collection::<WalkRequest>("walk_requests")
            .find_one(
                doc! {"_id": {"$toObjectId": id}},
                FindOneOptions::builder()
                    .projection(WALK_REQUEST_PROJECTION.clone())
                    .build(),
            )
            .await?
            .ok_or(Error::msg("walk request not found"))
    }

    async fn query_walk_requests(
        &self,
        query: WalkRequestQuery,
        sort_by: Option<SortBy>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<WalkRequest>, Error> {
        let mut q = doc! {};
        if let Some(ids) = query.dog_ids_includes_any {
            q.insert("dog_ids", doc! {"$elemMatch": {"$in": ids }});
        }
        if let Some(ids) = query.dog_ids_includes_all {
            q.insert("dog_ids", doc! {"$all": ids });
        }
        if let Some(accepted_by) = query.accepted_by {
            q.insert("accepted_by", accepted_by);
        }
        if let Some(nearby) = query.nearby {
            if nearby.len() != 3 {
                return Err(Error::msg("nearby query must be of length 3"));
            }
            let mut pipeline = vec![
                doc! {
                    "$geoNear": {
                        "near": { "type": "Point", "coordinates": [nearby[0], nearby[1]] },
                        "distanceField": "distance",
                        "maxDistance": nearby[2],
                        "spherical": true,
                        "query": q,
                        "includeLocs": "location",
                    }
                },
                doc! { "$project": WALK_REQUEST_PROJECTION.clone() },
            ];
            if let Some(pagination) = pagination {
                pipeline.push(doc! {
                    "$skip": (pagination.page - 1) * pagination.size
                });
                pipeline.push(doc! {
                    "$limit": pagination.size
                });
            }
            if let Some(sort_by) = sort_by {
                pipeline.push(doc! {
                    "$sort": {sort_by.field: if sort_by.order == Order::Asc { 1 } else { - 1} }
                })
            }
            return self
                .db
                .collection::<WalkRequest>("walk_requests")
                .aggregate(pipeline, None)
                .await?
                .map(|res| match res {
                    Err(e) => Err(Error::from(e)),
                    Ok(doc) => from_document::<WalkRequest>(doc).map_err(Error::from),
                })
                .try_collect::<Vec<WalkRequest>>()
                .await;
        }
        self.db
            .collection::<WalkRequest>("walk_requests")
            .find(
                q,
                FindOptions::builder()
                    .projection(WALK_REQUEST_PROJECTION.clone())
                    .limit(pagination.as_ref().map(|p| p.size))
                    .skip(
                        pagination
                            .as_ref()
                            .map(|p| (p.page as u64 - 1) * p.size as u64),
                    )
                    .sort(
                        sort_by.map(|s| doc! {s.field: if s.order == Order::Asc { 1 } else { - 1}}),
                    )
                    .build(),
            )
            .await?
            .try_collect::<Vec<WalkRequest>>()
            .await
            .map_err(|e| e.into())
    }

    async fn update_walk_request(
        &self,
        id: &str,
        request: WalkRequestUpdate,
    ) -> Result<u64, Error> {
        let mut set = doc! {};
        if let Some(dog_ids) = request.dog_ids {
            set.insert("dog_ids", dog_ids);
        }
        if let Some(accepted_by) = request.accepted_by {
            set.insert("accepted_by", accepted_by);
        }
        if let Some(latitude) = request.latitude {
            set.insert("latitude", latitude);
        }
        if let Some(longitude) = request.longitude {
            set.insert("longitude", longitude);
        }
        if let Some(should_start_after) = request.should_start_after {
            set.insert("should_start_after", should_start_after);
        }
        if let Some(should_start_before) = request.should_start_before {
            set.insert("should_start_before", should_start_before);
        }
        if let Some(should_end_before) = request.should_end_before {
            set.insert("should_end_before", should_end_before);
        }
        if let Some(should_end_after) = request.should_end_after {
            set.insert("should_end_after", should_end_after);
        }
        Ok(self
            .db
            .collection::<Document>("walk_requests")
            .update_one(doc! {"_id": {"$toObjectId": id}}, doc! {"$set": set}, None)
            .await
            .map_err(Error::from)?
            .modified_count)
    }
}
