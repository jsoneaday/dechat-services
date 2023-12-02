use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct PostWithProfileQueryResult {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub chain_asset_id: String,
    pub chain_id: i64,
    pub message: Option<String>,
    pub image: Option<Vec<u8>>,
    pub user_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Option<Vec<u8>>,
    pub respondee_post_id: Option<i64>    
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct PostWithProfileAndShareeQueryResult {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub chain_asset_id: String,
    pub chain_id: i64,
    pub message: Option<String>,
    pub image: Option<Vec<u8>>,
    pub user_id: i64,
    pub user_name: String,
    pub full_name: String,
    pub avatar: Option<Vec<u8>>,
    pub sharee_post_id: Option<i64>,
    pub sharee_post_updated_at: Option<DateTime<Utc>>,
    pub sharee_post_chain_asset_id: String,
    pub sharee_post_chain_id: i64,
    pub sharee_post_message: Option<String>,   
    pub sharee_post_image: Option<Vec<u8>>,
    pub sharee_post_user_id: Option<i64>,
    pub sharee_post_user_name: Option<String>,
    pub sharee_post_full_name: Option<String>,
    pub sharee_post_avatar: Option<Vec<u8>>
}