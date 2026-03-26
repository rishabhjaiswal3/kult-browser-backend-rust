use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
    Collection, Database,
};

use crate::config::CONFIG;
use crate::moments::creators::model::CreatorAggregate;

#[derive(Clone)]
pub struct CreatorsRepository {
    moments_collection: Collection<Document>,
}

impl CreatorsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            moments_collection: db.collection(&CONFIG.db.moments_collection),
        }
    }

    pub async fn find_creator(&self, wallet: &str) -> Result<Option<CreatorAggregate>, String> {
        let normalized_wallet = wallet.trim().to_lowercase();
        let mut pipeline = self.base_pipeline();
        pipeline.insert(0, doc! { "$match": { "playerWalletAddress": &normalized_wallet } });

        let mut cursor = self
            .moments_collection
            .aggregate(pipeline)
            .await
            .map_err(|e| e.to_string())?;

        let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? else {
            return Ok(None);
        };

        mongodb::bson::from_document(doc)
            .map(Some)
            .map_err(|e| e.to_string())
    }

    pub async fn list_leaderboard(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<CreatorAggregate>, String> {
        let skip = ((page.saturating_sub(1)) * page_size) as i64;
        let mut pipeline = self.base_pipeline();
        pipeline.push(doc! {
            "$sort": {
                "totalScore": -1,
                "totalSocialLikes": -1,
                "totalMomentLikes": -1,
                "totalMomentComments": -1,
                "walletAddress": 1
            }
        });
        pipeline.push(doc! { "$skip": skip });
        pipeline.push(doc! { "$limit": page_size as i64 });

        let cursor = self
            .moments_collection
            .aggregate(pipeline)
            .await
            .map_err(|e| e.to_string())?;

        let docs: Vec<Document> = cursor.try_collect().await.map_err(|e| e.to_string())?;
        docs.into_iter()
            .map(mongodb::bson::from_document::<CreatorAggregate>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub async fn count_creators(&self) -> Result<u64, String> {
        self.moments_collection
            .distinct("playerWalletAddress", doc! {})
            .await
            .map(|values| values.len() as u64)
            .map_err(|e| e.to_string())
    }

    pub async fn find_rank(&self, wallet: &str) -> Result<Option<u64>, String> {
        let normalized_wallet = wallet.trim().to_lowercase();
        let mut pipeline = self.base_pipeline();
        pipeline.push(doc! {
            "$sort": {
                "totalScore": -1,
                "totalSocialLikes": -1,
                "totalMomentLikes": -1,
                "totalMomentComments": -1,
                "walletAddress": 1
            }
        });
        pipeline.push(doc! {
            "$project": {
                "_id": 0,
                "walletAddress": 1
            }
        });

        let mut cursor = self
            .moments_collection
            .aggregate(pipeline)
            .await
            .map_err(|e| e.to_string())?;

        let mut rank = 1_u64;
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let Some(Bson::String(wallet_address)) = doc.get("walletAddress") else {
                rank += 1;
                continue;
            };

            if wallet_address == &normalized_wallet {
                return Ok(Some(rank));
            }
            rank += 1;
        }

        Ok(None)
    }

    fn base_pipeline(&self) -> Vec<Document> {
        vec![
            doc! {
                "$group": {
                    "_id": "$playerWalletAddress",
                    "totalMoments": { "$sum": 1 },
                    "totalMomentLikes": { "$sum": "$numLikes" },
                    "totalMomentComments": { "$sum": "$numComments" }
                }
            },
            doc! {
                "$lookup": {
                    "from": &CONFIG.db.shared_posts_collection,
                    "let": { "wallet": "$_id" },
                    "pipeline": [
                        {
                            "$match": {
                                "$expr": {
                                    "$and": [
                                        { "$eq": ["$wallet_address", "$$wallet"] },
                                        { "$eq": ["$validation_status", "Valid"] }
                                    ]
                                }
                            }
                        },
                        {
                            "$group": {
                                "_id": Bson::Null,
                                "totalSocialLikes": { "$sum": "$num_likes" },
                                "validatedPostsCount": { "$sum": 1 }
                            }
                        }
                    ],
                    "as": "socialStats"
                }
            },
            doc! {
                "$unwind": {
                    "path": "$socialStats",
                    "preserveNullAndEmptyArrays": true
                }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "walletAddress": "$_id",
                    "totalMoments": "$totalMoments",
                    "totalMomentLikes": "$totalMomentLikes",
                    "totalMomentComments": "$totalMomentComments",
                    "totalSocialLikes": { "$ifNull": ["$socialStats.totalSocialLikes", 0] },
                    "validatedPostsCount": { "$ifNull": ["$socialStats.validatedPostsCount", 0] },
                    "successfulReferrals": { "$literal": 0 },
                    "totalScore": {
                        "$add": [
                            "$totalMomentLikes",
                            "$totalMomentComments",
                            { "$ifNull": ["$socialStats.totalSocialLikes", 0] },
                            0
                        ]
                    }
                }
            },
        ]
    }
}
