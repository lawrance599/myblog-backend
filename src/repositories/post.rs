use sqlx::types::Json;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool};
use tracing::instrument;

/// 存储的博文结构
#[allow(dead_code)]
#[derive(FromRow, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub tags: Json<Vec<String>>,
    first_publish: DateTime<Utc>,
    last_modify: DateTime<Utc>,
    pub count: i32,
}

/// PostService
/// 博文有关的数据库操作的封装
/// EXAMPLE:
/// ```compile_fail
/// use sqlx::{Pool, Postgres};
/// # use sqlx::types::chrono::{DateTime, Utc};
/// # use sqlx::types::Json;
/// # use crate::service::post::Post;
/// use crate::service::post::PostService;
///
/// let pool = sqlx::PgPool::connect("postgres://yixin:yixin@localhost:5432/test").await.unwrap();
/// let service = PostService::new(pool);
/// let post = service.get_post_by_id(1).await;
/// assert!(post.is_ok());
/// ```
pub struct PostReponsitory(Pool<sqlx::Postgres>);
impl PostReponsitory {
    pub fn new(pool: Pool<sqlx::Postgres>) -> PostReponsitory {
        tracing::info!("创建PostReponsitory成功");
        PostReponsitory(pool)
    }
    #[instrument(level = "debug", skip(self))]
    pub async fn find_by_id(&self, id: i32) -> Result<Post, sqlx::Error> {
        let _ = sqlx::query_as::<_, Post>("SELECT * FROM post WHERE id = $1")
            .bind(id)
            .fetch_one(&self.0)
            .await?;

        self.increase_count(id).await
    }
    #[instrument(level = "debug", skip(self))]
    pub async fn find_all(&self, limit: i32) -> Result<Vec<Post>, sqlx::Error> {
        assert!(limit > 0);
        let posts = sqlx::query_as::<_, Post>("SELECT * FROM post LIMIT $1")
            .bind(limit)
            .fetch_all(&self.0)
            .await?;
        let mut last_error: Option<sqlx::Error> = None;
        for post in posts.iter() {
            let _ = match self.increase_count(post.id).await {
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
    #[instrument(level = "debug", skip(self))]
    pub async fn insert_one(&self, title: String, tags: Vec<String>) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("INSERT INTO post (title, tags) VALUES ($1, $2) RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level = "debug", skip(self))]
    pub async fn update_one(
        &self,
        id: i32,
        title: String,
        tags: Vec<String>,
    ) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE post SET title = $1, tags = $2 WHERE id = $3 RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level = "trace", skip(self))]
    async fn increase_count(&self, id: i32) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE post SET count = count + 1 WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_by_id(&self, id: i32) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("DELETE FROM post WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.0)
            .await
    }

    /// 批量删除post
    ///
    /// # Arguments
    ///
    /// - `ids` (`Vec<i32>`) - 将要删除的post的id
    ///
    /// # Returns
    ///
    /// - `Result<Vec<Post>, sqlx::Error>` - 删除的post
    ///
    /// # Errors
    ///
    /// - `sqlx::Error` - 删除失败
    /// 只会返回最后一个错误
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::...;
    ///
    /// async {
    ///   let result = delete_posts(Vec::new([1, 2, 3])).await;
    /// };
    /// ```
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_many(&self, ids: Vec<i32>) -> Result<Vec<Post>, sqlx::Error> {
        let mut last_error: Option<sqlx::Error> = None;
        let mut posts = Vec::new();
        for id in ids.iter() {
            let _ = match self.delete_by_id(*id).await {
                Ok(post) => {
                    posts.push(post);
                }
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

    #[instrument(level = "debug", skip_all)]
    pub async fn find_by_tags(&self, tags: Vec<String>) -> Result<Vec<Post>, sqlx::Error> {
        let posts = sqlx::query_as::<_, Post>("SELECT * FROM post WHERE tags @> $1")
            .bind(Json(tags))
            .fetch_all(&self.0)
            .await?;
        Ok(posts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::lorem::zh_cn::{Sentence, Words};
    use fake::{Dummy, Fake, Faker};

    #[derive(Dummy)]
    struct FakePostInsert {
        #[dummy(faker = "Sentence(5..10)")]
        title: String,
        #[dummy(faker = "Words(2..4)")]
        tags: Vec<String>,
    }

    /// 生成一个假的 Post 实例
    fn generate_fake_post() -> FakePostInsert {
        let fake_post: FakePostInsert = Faker.fake();
        fake_post
    }

    /// 生成多个假的 Post 实例
    fn generate_fake_posts(count: usize) -> Vec<FakePostInsert> {
        (0..count).map(|_| generate_fake_post()).collect()
    }

    #[tokio::test]
    async fn test_generate_fake_post() {
        let post = generate_fake_post();
        assert!(!post.title.is_empty());
    }

    #[tokio::test]
    async fn test_generate_fake_posts() {
        let posts = generate_fake_posts(5);
        assert_eq!(posts.len(), 5);

        for post in posts {
            assert!(!post.title.is_empty());
        }
    }

    async fn generate_post_service() -> PostReponsitory {
        let pool = sqlx::PgPool::connect("postgres://yixin:yixin@localhost:5432/test")
            .await
            .unwrap();
        PostReponsitory::new(pool)
    }

    #[tokio::test]
    async fn test_insert_and_get_post() {
        let service = generate_post_service().await;
        let fake_post = generate_fake_post();
        let inserted_post = service
            .insert_one(fake_post.title.clone(), fake_post.tags.clone())
            .await;
        assert!(inserted_post.is_ok());
        let post = service.find_by_id(inserted_post.unwrap().id).await;
        assert!(post.is_ok());
        let post = post.unwrap();
        assert_eq!(post.title.clone(), fake_post.title);
        assert_eq!(post.tags.as_ref(), &fake_post.tags);
        assert!(service.delete_by_id(post.id).await.is_ok())
    }

    #[tokio::test]
    async fn test_query_count_works_ok() {
        let service = generate_post_service().await;
        let fake_post = generate_fake_post();
        let inserted_post = service
            .insert_one(fake_post.title.clone(), fake_post.tags.clone())
            .await;
        assert!(inserted_post.is_ok());
        let inserted_post = inserted_post.unwrap();
        assert_eq!(inserted_post.count, 0);
        let again_post = service.find_by_id(inserted_post.id).await;
        assert!(again_post.is_ok());
        let again_post = again_post.unwrap();
        assert_eq!(again_post.count, 1);
    }

    #[tokio::test]
    async fn test_get_all_posts() {
        let service = generate_post_service().await;
        let posts = generate_fake_posts(5);
        for post in posts {
            let inserted_post = service
                .insert_one(post.title.clone(), post.tags.clone())
                .await;
            assert!(inserted_post.is_ok());
        }
        let all_posts = service.find_all(10).await;
        assert!(all_posts.is_ok());
    }

    #[tokio::test]
    async fn test_update_post() {
        let service = generate_post_service().await;
        let fake_post = generate_fake_post();
        let inserted_post = service
            .insert_one(fake_post.title.clone(), fake_post.tags.clone())
            .await;
        assert!(inserted_post.is_ok());
        let inserted_post = inserted_post.unwrap();

        let updated_post = service
            .update_one(
                inserted_post.id,
                "new title".to_string(),
                vec!["new tag".to_string()],
            )
            .await;
        assert!(updated_post.is_ok());
        let updated_post = updated_post.unwrap();
        assert_eq!(updated_post.title, "new title");
        assert_eq!(updated_post.tags, vec!["new tag".to_string()].into());

        assert!(service.delete_by_id(updated_post.id).await.is_ok())
    }
}
