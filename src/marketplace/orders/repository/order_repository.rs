// src/marketplace/orders/repository/order_repository.rs

use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use mongodb::{Collection, Database};

use crate::config::CONFIG;
use crate::marketplace::orders::model::OrderModel;

#[derive(Clone)]
pub struct OrderRepository {
    collection: Collection<OrderModel>,
    raw_collection: Collection<Document>,
}

impl OrderRepository {
    pub fn new(db: &Database) -> Self {
        let name = &CONFIG.db.marketplace_orders_collection;
        Self {
            collection: db.collection(name),
            raw_collection: db.collection(name),
        }
    }

    /// Create a new order.
    pub async fn create(&self, order: OrderModel) -> Result<ObjectId, String> {
        let mut doc = mongodb::bson::to_document(&order).map_err(|e| e.to_string())?;
        doc.insert("createdAt", DateTime::now());

        let result = self
            .raw_collection
            .insert_one(doc)
            .await
            .map_err(|e| e.to_string())?;

        result
            .inserted_id
            .as_object_id()
            .ok_or_else(|| "Failed to get inserted ID".to_string())
    }

    /// Find order by ID.
    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<OrderModel>, String> {
        self.collection
            .find_one(doc! { "_id": id })
            .await
            .map_err(|e| e.to_string())
    }

    /// Find order by backend orderId.
    pub async fn find_by_order_id(&self, order_id: &str) -> Result<Option<OrderModel>, String> {
        self.collection
            .find_one(doc! { "orderId": order_id })
            .await
            .map_err(|e| e.to_string())
    }

    /// Find orders by player ID (paginated).
    pub async fn find_by_player(
        &self,
        player_id: &str,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<OrderModel>, String> {
        let cursor = self
            .collection
            .find(doc! { "playerId": player_id })
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Count orders by player ID.
    pub async fn count_by_player(&self, player_id: &str) -> Result<u64, String> {
        self.collection
            .count_documents(doc! { "playerId": player_id })
            .await
            .map_err(|e| e.to_string())
    }

    /// Find all orders (admin, paginated).
    pub async fn find_all(
        &self,
        skip: u64,
        limit: i64,
    ) -> Result<Vec<OrderModel>, String> {
        let cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "createdAt": -1 })
            .skip(skip)
            .limit(limit)
            .await
            .map_err(|e| e.to_string())?;

        cursor.try_collect().await.map_err(|e| e.to_string())
    }

    /// Count all orders.
    pub async fn count_all(&self) -> Result<u64, String> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| e.to_string())
    }

    /// Update order status.
    pub async fn update_status(
        &self,
        id: &ObjectId,
        status: &str,
    ) -> Result<Option<OrderModel>, String> {
        self.collection
            .find_one_and_update(
                doc! { "_id": id },
                doc! { "$set": { "status": status } },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }

    /// Confirm an order by backend orderId with tx hash and completed status.
    pub async fn confirm_by_order_id(
        &self,
        order_id: &str,
        tx_hash: &str,
    ) -> Result<Option<OrderModel>, String> {
        self.collection
            .find_one_and_update(
                doc! { "orderId": order_id },
                doc! {
                    "$set": {
                        "status": "completed",
                        "txHash": tx_hash,
                    }
                },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
            .map_err(|e| e.to_string())
    }
}
