use sqlx::{FromRow, Pool};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
#[derive(FromRow)]
pub struct Post{
    id: i32,
    title: String,
    tags: Json<Vec<String>>,
    first_publish: DateTime<Utc>,
    last_modify: DateTime<Utc>,
    query_count: i32,
}

/// PostService
/// 博文有关的数据库操作的封装
pub struct PostService<'a>(&'a Pool<sqlx::Postgres>);
impl PostService<'_> {
    pub fn new(pool:&'_  Pool<sqlx::Postgres>) -> PostService<'_> {
        PostService(pool)
    }
    pub async fn get_post_by_id(&self, id: i32) -> Result<Post, sqlx::Error> {
        let _ = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_one(&self.0.clone())
            .await?;

        self.increase_query_count(id).await
    }
    pub async fn get_all_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts")
            .fetch_all(&self.0.clone())
            .await?;
        let mut last_error: Option<sqlx::Error> = None;
        for post in posts.iter() {
            let _ = match self.increase_query_count(post.id).await {
                Ok(_) => {}
                Err(e) => {
                    last_error = Some(e);
                }
            };
        }
        if last_error.is_none() {
            Ok(posts)
        } else {
            Err(last_error.unwrap())
        }
    }
    pub async fn create_post(&self, title: String, tags: Vec<String>) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("INSERT INTO posts (title, tags) VALUES ($1, $2) RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .fetch_one(&self.0.clone())
            .await
    }

    pub async fn update_post(&self, id: i32, title: String, tags: Vec<String>) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE posts SET title = $1, tags = $2 WHERE id = $3 RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .bind(id)
            .fetch_one(&self.0.clone())
            .await
    }

    async fn increase_query_count(&self, id: i32) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE posts SET query_count = query_count + 1 WHERE id =  RETURNING *")
            .bind(id)
            .fetch_one(&self.0.clone())
            .await
    }
}
