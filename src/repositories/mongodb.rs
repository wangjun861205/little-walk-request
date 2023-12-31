use mongodb::bson::oid::ObjectId;
use mongodb::bson::{from_document, Document};
use mongodb::options::FindOneAndUpdateOptions;
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Database,
};

use crate::core::entities::WalkRequest;
use crate::core::repository::{Order, Pagination, Repository, SortBy, WalkingLocationCreate};
use crate::core::repository::{WalkRequestCreate, WalkRequestQuery, WalkRequestUpdate};
use anyhow::Error;
use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use little_walk_dog::core::entities::Dog;
use std::str::FromStr;

impl WalkRequest {
    pub fn projection() -> Document {
        doc! {
            "id": {"$toString": "$_id"},
            "dogs": Dog::projection(),
            "should_start_after": {"$dateToString": {"date":"$should_start_after", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "should_start_before": {"$dateToString": {"date":"$should_start_before", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "should_end_after": {"$dateToString": {"date":"$should_end_after", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "should_end_before": {"$dateToString": {"date":"$should_end_before", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "longitude": { "$arrayElemAt": [ "$location.coordinates", 0]},
            "latitude": { "$arrayElemAt": [ "$location.coordinates", 1]},
            "distance": "$distance",
            "canceled_at": {"$dateToString": {"date":"$canceled_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "accepted_by": "$accepted_by",
            "accepted_at": {"$dateToString": {"date":"$accepted_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "started_at": {"$dateToString": {"date":"$started_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "finished_at": {"$dateToString": {"date":"$finished_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "status": {
                "$switch": {
                    "branches": [
                        {"case": {"$ne": [{"$ifNull": ["$canceled_at", null]}, null]}, "then": "Canceled" },
                        {"case": {"$ne": [{"$ifNull": ["$accepted_at", null]}, null]}, "then": "Accepted" },
                        {"case": {"$ne": [{"$ifNull": ["$started_at", null]}, null]}, "then": "Started" },
                        {"case": {"$ne": [{"$ifNull": ["$finished_at", null]}, null]}, "then": "Finished" },
                    ],
                    "default": "Waiting"
                }
            },
            "acceptances": "$acceptances",
            "created_at": {"$dateToString": {"date":"$created_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
            "updated_at": {"$dateToString": {"date":"$updated_at", "format": "%Y-%m-%dT%H:%M:%S.%LZ"}},
        }
    }
}

impl TryFrom<WalkRequestQuery> for Document {
    type Error = Error;
    fn try_from(value: WalkRequestQuery) -> Result<Self, Self::Error> {
        let mut q = doc! {};
        if let Some(id) = value.id {
            q.insert("_id", ObjectId::from_str(&id)?);
        }
        if let Some(ids) = value.dog_ids_includes_any {
            q.insert("dogs.id", doc! {"$elemMatch": {"$in": ids }});
        }
        if let Some(ids) = value.dog_ids_includes_all {
            q.insert("dogs.id", doc! {"$all": ids });
        }
        if let Some(accepted_by) = value.accepted_by {
            q.insert("accepted_by", accepted_by);
        }
        if let Some(accepted_by_neq) = value.accepted_by_neq {
            q.insert("accepted_by", doc! {"$ne": accepted_by_neq });
        }
        if let Some(accepted_by_is_null) = value.accepted_by_is_null {
            if accepted_by_is_null {
                q.insert("accepted_by", doc! {"$eq": null});
            } else {
                q.insert("accepted_by", doc! {"$neq": null});
            }
        }
        if let Some(acceptances_includes_all) = value.acceptances_includes_all {
            q.insert("acceptances", doc! {"$all": acceptances_includes_all });
        }
        if let Some(acceptances_includes_any) = value.acceptances_includes_any {
            q.insert(
                "acceptances",
                doc! {"$elemMatch": {"$in": acceptances_includes_any }},
            );
        }
        if let Some(nearby) = value.nearby {
            if nearby.len() != 3 {
                return Err(anyhow::anyhow!("Invalid nearby query, expect [f64;3]"));
            }
            return Ok(doc! {
                "$geoNear": {
                    "near": { "type": "Point", "coordinates": [nearby[0], nearby[1]] },
                    "distanceField": "distance",
                    "maxDistance": nearby[2],
                    "spherical": true,
                    "query": q,
                    "includeLocs": "location",
                }
            });
        }
        if let Some(created_by) = value.created_by {
            q.insert("created_by", created_by);
        }
        Ok(q)
    }
}

impl From<WalkRequestUpdate> for Document {
    fn from(update: WalkRequestUpdate) -> Self {
        let mut set = doc! {};
        if let Some(dogs) = update.dogs {
            set.insert("dogs", dogs);
        }
        if let Some(accepted_by) = update.accepted_by {
            set.insert("accepted_by", accepted_by);
        }
        if let Some(accepted_at) = update.accepted_at {
            set.insert("accepted_at", accepted_at);
        }
        if let Some(latitude) = update.latitude {
            set.insert("latitude", latitude);
        }
        if let Some(longitude) = update.longitude {
            set.insert("longitude", longitude);
        }
        if let Some(should_start_after) = update.should_start_after {
            set.insert("should_start_after", should_start_after);
        }
        if let Some(should_start_before) = update.should_start_before {
            set.insert("should_start_before", should_start_before);
        }
        if let Some(should_end_before) = update.should_end_before {
            set.insert("should_end_before", should_end_before);
        }
        if let Some(should_end_after) = update.should_end_after {
            set.insert("should_end_after", should_end_after);
        }
        if let Some(add_to_acceptances) = update.add_to_acceptances {
            set.insert("$addToSet", doc! {"acceptances": add_to_acceptances});
        }
        if let Some(started_at) = update.started_at {
            set.insert("started_at", started_at);
        }
        if let Some(finished_at) = update.finished_at {
            set.insert("finished_at", finished_at);
        }
        let mut pull = doc! {};
        if let Some(remove_from_acceptances) = update.remove_from_acceptances {
            pull.insert("acceptances", remove_from_acceptances);
        }
        let mut unset = doc! {};
        if update.unset_accepted_by {
            unset.insert("accepted_by", "");
        }
        if update.unset_accepted_at {
            unset.insert("accepted_at", "");
        }
        doc! {"$set": set, "$unset": unset, "$pull": pull}
    }
}

impl From<WalkRequestCreate> for Document {
    fn from(value: WalkRequestCreate) -> Self {
        doc! {
            "dogs": value.dogs,
            "should_start_after": value.should_start_after,
            "should_start_before": value.should_start_before,
            "should_end_before": value.should_end_before,
            "should_end_after": value.should_end_after,
            "location": { "type": "Point", "coordinates": [value.longitude, value.latitude] },
            "created_by": value.created_by,
            "created_at": Utc::now(),
            "updated_at": Utc::now(),
        }
    }
}

impl<'a> From<WalkingLocationCreate<'a>> for Document {
    fn from(value: WalkingLocationCreate) -> Self {
        doc! {
            "walk_request_id": value.walk_request_id,
            "longitude": value.longitude,
            "latitude": value.latitude,
            "created_at": Utc::now(),
            "updated_at": Utc::now(),
        }
    }
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
            .collection::<Document>("walk_requests")
            .insert_one(Document::from(request), None)
            .await?;
        Ok(inserted.inserted_id.to_string())
    }

    async fn get_walk_request(&self, id: &str) -> Result<WalkRequest, Error> {
        self.db
            .collection::<WalkRequest>("walk_requests")
            .find_one(
                doc! {"_id": ObjectId::from_str(id)?},
                FindOneOptions::builder()
                    .projection(WalkRequest::projection())
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
        if query.nearby.is_some() {
            let mut pipeline = vec![
                Document::try_from(query)?,
                doc! { "$project": WalkRequest::projection() },
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
                Document::try_from(query)?,
                FindOptions::builder()
                    .projection(WalkRequest::projection())
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
    ) -> Result<WalkRequest, Error> {
        self.db
            .collection("walk_requests")
            .find_one_and_update(
                doc! {"_id": ObjectId::from_str(id)?},
                Document::from(request),
                FindOneAndUpdateOptions::builder()
                    .return_document(Some(mongodb::options::ReturnDocument::After))
                    .projection(WalkRequest::projection())
                    .build(),
            )
            .await?
            .ok_or(Error::msg("代遛请求不存在"))
    }

    async fn update_walk_request_by_query(
        &self,
        query: WalkRequestQuery,
        update: WalkRequestUpdate,
    ) -> Result<WalkRequest, Error> {
        self.db
            .collection("walk_requests")
            .find_one_and_update(
                Document::try_from(query)?,
                Document::from(update),
                FindOneAndUpdateOptions::builder()
                    .return_document(Some(mongodb::options::ReturnDocument::After))
                    .projection(WalkRequest::projection())
                    .build(),
            )
            .await?
            .ok_or(Error::msg("代遛请求不存在"))
    }

    async fn update_walk_requests_by_query(
        &self,
        query: WalkRequestQuery,
        update: WalkRequestUpdate,
    ) -> Result<u64, Error> {
        Ok(self
            .db
            .collection::<Document>("walk_requests")
            .update_many(Document::try_from(query)?, Document::from(update), None)
            .await?
            .modified_count)
    }

    async fn create_walking_location<'a>(
        &self,
        create: WalkingLocationCreate<'a>,
    ) -> Result<String, Error> {
        self.db
            .collection("walking_locations")
            .insert_one(Document::from(create), None)
            .await
            .map_err(|e| Error::new(e).context("创建Walking定位失败"))
            .map(|r| r.inserted_id.to_string())
    }
}
