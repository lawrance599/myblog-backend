use crate::repositories::comment::Comment;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize)]
pub struct CommentCreate {
    pub post_id: i32,
    pub author: String,
    pub content: String,
    pub parent_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct CommentUpdate {
    pub content: String,
}

#[derive(Serialize)]
pub struct CommentRead {
    pub id: i32,
    pub post_id: i32,
    pub author: String,
    pub content: String,
    pub created_at: String,
    pub parent_id: Option<i32>,
}

impl From<Comment> for CommentRead {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            author: value.author,
            content: value.content,
            created_at: value.created_at.to_string(),
            parent_id: value.parent_id,
        }
    }
}
