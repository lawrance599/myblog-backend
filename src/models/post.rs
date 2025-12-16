use serde::{Deserialize, Serialize};

use crate::repositories::post::PostMeta;
#[derive(Default)]
pub struct PostCreate {
    pub title: String,
    pub tags: Vec<String>,
    pub content: Vec<u8>,
}
#[derive(Serialize)]
pub struct PostId {
    pub id: i32,
}
impl PostId {
    pub fn new(id: i32) -> Self {
        return Self { id };
    }
}
#[derive(Deserialize)]
pub struct RowTags {
    pub tags: String,
}
pub struct Tags {
    pub tags: Vec<String>,
}
impl TryFrom<RowTags> for Tags {
    type Error = String;
    fn try_from(value: RowTags) -> Result<Self, Self::Error> {
        return Ok(Self {
            tags: value.tags.split(',').map(|s| s.to_string()).collect(),
        });
    }
}
#[derive(Serialize)]
pub struct PostMetaRead {
    id: i32,
    title: String,
    tags: Vec<String>,
    count: i32,
}
impl From<PostMeta> for PostMetaRead {
    fn from(value: PostMeta) -> Self {
        return Self {
            id: value.id,
            title: value.title,
            tags: value.tags.0,
            count: value.count,
        };
    }
}
#[derive(Serialize)]
pub struct Post {
    id: i32,
    title: String,
    tags: Vec<String>,
    content: String,
    count: i32,
}
impl Post {
    pub fn with_content(meta: PostMeta, content: String) -> Self {
        return Self {
            id: meta.id,
            title: meta.title,
            tags: meta.tags.0,
            content,
            count: meta.count,
        };
    }
}
