use chrono::{ DateTime, Utc };
use serde::Deserialize;
use sqlx::FromRow;
use fake::{
    Fake,
    faker::internet::en::DomainSuffix
};
use fake::faker::lorem::en::Sentence;
use fake::faker::company::en::CompanyName;
use std::ops::Range ;
use common::file_utils::get_avatar_buffer;

pub const SUI_CHAIN_ID: i64 = 1;
pub const PUBLIC_GROUP_TYPE: i32 = 1;
pub const CIRCLE_GROUP_TYPE: i32 = 2;
#[allow(unused)]
const JPEG_SIGNATURE: [u8; 2] = [0xFF, 0xD8];
#[allow(unused)]
const JPEG_END_SIGNATURE: [u8; 2] = [0xFF, 0xD9];

#[allow(unused)]
#[derive(Deserialize, FromRow)]
pub struct MessageResponse {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub original_msg_id: i64,
    pub responding_msg_id: i64,
}

#[derive(Debug, Clone)]
pub enum FixtureError {
    MissingData(String),
    QueryFailed(String),
}
impl std::error::Error for FixtureError {}
impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingData(msg) => write!(f, "{}", msg),
            Self::QueryFailed(msg) => write!(f, "{}", msg),
        }
    }
}

#[allow(unused)]
fn is_jpeg(avatar: Vec<u8>) -> bool {
    let mut is_valid = false;
    if avatar.len() >= 2 && &avatar[0..2] == JPEG_SIGNATURE {
        let end_offset = avatar.len() - 2;
        if &avatar[end_offset..] == JPEG_END_SIGNATURE {
            println!("The avatar data is a valid JPEG image.");
            is_valid = true;
        } else {
            println!("The avatar data does not have a valid JPEG end signature.");
        }
    } else {
        println!("The avatar data does not have a valid JPEG signature.");
    }
    is_valid
}

pub fn get_fake_message_body(prefix: Option<String>) -> String {
    let mut body: String = match prefix {
        Some(pref) => pref,
        None => "".to_string(),
    };

    for _ in [..4] {
        let random_sentence: String = Sentence(Range { start: 5, end: 6 }).fake();
        body = format!("{}. {}", body, random_sentence);
    }
    body
}

pub fn get_fake_main_url() -> String {
    let mut domain = CompanyName().fake::<String>();
    domain.retain(|str| !str.is_whitespace());
    format!("https://{}.{}", domain, DomainSuffix().fake::<String>())
}

pub fn get_profile_avatar() -> Vec<u8> {
    let file_name = "profile.jpeg".to_string();
    let file_path = format!("src/common_tests/{}", file_name);

    get_avatar_buffer(&file_path)
}