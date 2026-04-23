use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::Type;


#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "post_status", rename_all = "PascalCase")]  // 让 sqlx 中的 post_status 类型与 Rust 中的 PostStatus 枚举对应
enum PostStatus {
    Draft,
    Published,
    Archived,
}


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    id: i32,
    title: String,
    content: String,
    author_id: i32,
    created_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    edited_at: Option<DateTime<Utc>>,
    status: PostStatus,
}