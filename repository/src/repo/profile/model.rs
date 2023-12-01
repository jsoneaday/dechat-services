use chrono::prelude::*;
use serde::{ Deserialize, Serialize };
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct ProfileQueryResult {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub chain_id: String,
    pub user_name: String,
    pub full_name: String,
    pub description: String,
    pub main_url: Option<String>,
    pub avatar: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProfileCreate {
    pub chain_id: String,
    pub user_name: String,
    pub full_name: String,
    pub description: String,
    pub main_url: Option<String>,
    pub avatar: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProfileUpdate {
    pub full_name: String,
    pub description: String,
    pub main_url: Option<String>,
    pub avatar: Option<Vec<u8>>,
}