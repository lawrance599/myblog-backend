use sqlx::{FromRow, Pool};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use tracing::{Level, event, instrument};
#[allow(dead_code)]
#[derive(FromRow, Debug)]
pub struct Post{
    id: i32,
    title: String,
    tags: Json<Vec<String>>,
    first_publish: DateTime<Utc>,
    last_modify: DateTime<Utc>,
    count: i32,
}


/// PostService
/// 博文有关的数据库操作的封装
pub struct PostService(Pool<sqlx::Postgres>);
impl PostService {

    pub fn new(pool:  Pool<sqlx::Postgres>) -> PostService {
        event!(Level::INFO, "PostService init");
        PostService(pool)
    }
    #[instrument(level="debug", skip(self))]
    pub async fn get_post_by_id(&self, id: i32) -> Result<Post, sqlx::Error> {
        let _ = sqlx::query_as::<_, Post>("SELECT * FROM post WHERE id = $1")
            .bind(id)
            .fetch_one(&self.0)
            .await?;

        self.increase_count(id).await
    }
    #[instrument(level="debug", skip(self))]
    pub async fn get_all_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        let posts = sqlx::query_as::<_, Post>("SELECT * FROM post")
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
    #[instrument(level="debug", skip_all, fields(id))]
    pub async fn create_post(&self, title: String, tags: Vec<String>) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("INSERT INTO post (title, tags) VALUES ($1, $2) RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level="debug", skip_all, fields(id))]
    pub async fn update_post(&self, id: i32, title: String, tags: Vec<String>) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE post SET title = $1, tags = $2 WHERE id = $3 RETURNING *")
            .bind(title)
            .bind(Json(tags))
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level="trace", skip(self))]
    async fn increase_count(&self, id: i32) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("UPDATE post SET count = count + 1 WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
    #[instrument(level="debug", skip(self))]
    pub async fn delete_post_by_id(&self, id: i32) -> Result<Post, sqlx::Error> {
        sqlx::query_as::<_, Post>("DELETE FROM post WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Dummy, Fake, Faker};

    #[derive(Dummy)]
    struct FakePostInsert {
        title: String,
        tags: Vec<String>,
    }

    /// 生成一个假的 Post 实例
    fn generate_fake_post() -> FakePostInsert {
        let fake_post: FakePostInsert = Faker.fake();
        fake_post.into()
    }

    /// 生成多个假的 Post 实例
    fn generate_fake_posts(count: usize) -> Vec<FakePostInsert> {
        (0..count).map(|_| generate_fake_post()).collect()
    }


    #[tokio::test]
    async fn test_generate_fake_post() {
        let post = generate_fake_post();
        
        // 验证生成的数据
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

    async fn generate_post_service() -> PostService{
        let pool = sqlx::PgPool::connect("postgres://yixin:yixin@localhost:5432/test").await.unwrap();
        PostService::new(pool)
    }

    #[tokio::test]
    async fn test_insert_and_get_post() {
        let service = generate_post_service().await;
        let fake_post = generate_fake_post();
        let inserted_post = service.create_post(fake_post.title.clone(), fake_post.tags.clone()).await;
        assert!(inserted_post.is_ok());
        let post = service.get_post_by_id(inserted_post.unwrap().id).await;
        assert!(post.is_ok());
        let post = post.unwrap();
        assert_eq!(post.title.clone(), fake_post.title);
        assert_eq!(post.tags.as_ref(), &fake_post.tags);
        assert!(service.delete_post_by_id(post.id).await.is_ok())
    }

    #[tokio::test]
    async fn test_query_count_works_ok(){
        let service = generate_post_service().await;
        let fake_post = generate_fake_post();
        let inserted_post = service.create_post(fake_post.title.clone(), fake_post.tags.clone()).await;
        assert!(inserted_post.is_ok());
        let inserted_post = inserted_post.unwrap();
        assert_eq!(inserted_post.count, 0);
        let again_post = service.get_post_by_id(inserted_post.id).await;
        assert!(again_post.is_ok());
        let again_post = again_post.unwrap();
        assert_eq!(again_post.count, 1);
        assert!(service.delete_post_by_id(again_post.id).await.is_ok())
    }

}