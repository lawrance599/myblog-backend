use crate::repositories::ReponsitoryError;
use crate::repositories::post::{PostMeta, PostMetaCreate, PostMetaReponsitory, PostMetaUpdate};
use async_trait::async_trait;
use serde_json;
use sqlx::Pool;
use sqlx::Row;
use sqlx::types::Json;
use tracing::{Level, event, instrument};

pub struct SqlxReponsitory(Pool<sqlx::Postgres>);

impl SqlxReponsitory {
    pub fn new(pool: Pool<sqlx::Postgres>) -> SqlxReponsitory {
        tracing::info!("创建PostRepository成功");
        SqlxReponsitory(pool)
    }

    #[instrument(level = "trace", skip(self))]
    async fn increase_count(&self, id: i32) -> Result<PostMeta, sqlx::Error> {
        sqlx::query_as::<_, PostMeta>("UPDATE post SET count = count + 1 WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.0)
            .await
    }
}

#[async_trait]
impl PostMetaReponsitory for SqlxReponsitory {
    #[instrument(name = "PostMetaReponsitory::list_all", level = "debug", skip(self))]
    async fn list_pagenigation(
        &self,
        start_id: i32,
        page_size: i32,
    ) -> Result<Vec<PostMeta>, ReponsitoryError> {
        event!(
            Level::DEBUG,
            start_id = start_id,
            page_size = page_size,
            "开始分页查询文章元数据"
        );

        let posts = sqlx::query_as::<_, PostMeta>(
            "SELECT id, title, tags, first_publish, last_modify, count FROM post LIMIT id > $1 limit $2",
        )
        .bind(start_id)
        .bind(page_size)
        .fetch_all(&self.0)
        .await?;

        event!(Level::DEBUG, post_count = posts.len(), "成功查询文章元数据");

        // 增加每个 post 的计数
        for post in posts.iter() {
            let _ = self.increase_count(post.id).await?;
        }

        event!(Level::DEBUG, "成功更新所有文章的访问计数");
        Ok(posts)
    }

    #[instrument(name = "PostMetaReponsitory::find_by_id", level = "debug", skip(self))]
    async fn find_by_id(&self, id: i32) -> Result<PostMeta, ReponsitoryError> {
        event!(Level::DEBUG, post_id = id, "开始根据ID查询文章元数据");

        let post = sqlx::query_as::<_, PostMeta>(
            "SELECT id, title, tags, first_publish, last_modify, count FROM post WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.0)
        .await?;

        event!(Level::DEBUG, post_id = id, title = %post.title, "成功查询文章元数据");
        Ok(post)
    }

    #[instrument(
        name = "PostMetaReponsitory::find_by_keywords",
        level = "debug",
        skip(self)
    )]
    async fn find_by_keywords(
        &self,
        keywords: &[String],
    ) -> Result<Vec<PostMeta>, ReponsitoryError> {
        if keywords.is_empty() {
            event!(Level::DEBUG, "关键词列表为空，返回空结果");
            return Ok(vec![]);
        }

        event!(Level::DEBUG, keywords_count = keywords.len(), keywords = ?keywords, "开始根据关键词查询文章");

        // 构建正确的tsquery格式，每个关键词用 & 连接
        let query_string = keywords
            .iter()
            .map(|keyword| format!("'{}'", keyword.replace("'", "''")))
            .collect::<Vec<_>>()
            .join(" & ");

        let posts = sqlx::query_as::<_, PostMeta>(
            "SELECT * FROM post WHERE kw @@ to_tsquery('simple', $1)",
        )
        .bind(query_string)
        .fetch_all(&self.0)
        .await?;

        event!(
            Level::DEBUG,
            post_count = posts.len(),
            "成功根据关键词查询文章"
        );
        Ok(posts)
    }

    #[instrument(name = "PostMetaReponsitory::find_by_tags", level = "debug", skip_all)]
    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<PostMeta>, ReponsitoryError> {
        if tags.is_empty() {
            event!(Level::DEBUG, "标签列表为空，返回空结果");
            return Ok(vec![]);
        }

        event!(Level::DEBUG, tags_count = tags.len(), tags = ?tags, "开始根据标签查询文章");

        let posts = sqlx::query_as::<_, PostMeta>("SELECT * FROM post WHERE tags @> $1::jsonb")
            .bind(serde_json::to_value(tags).unwrap())
            .fetch_all(&self.0)
            .await?;

        event!(
            Level::DEBUG,
            post_count = posts.len(),
            "成功根据标签查询文章"
        );
        Ok(posts)
    }

    #[instrument(name = "PostMetaReponsitory::add", level = "debug", skip_all)]
    async fn add(&self, post: PostMetaCreate) -> Result<PostMeta, ReponsitoryError> {
        let PostMetaCreate { title, tags, kw } = post;

        event!(Level::DEBUG, title = %title, tags_count = tags.len(), keywords_count = kw.len(), "开始创建文章元数据");

        let new_post = sqlx::query_as::<_, PostMeta>(
            r#"INSERT INTO
            post (title, tags, kw)
            VALUES ($1, $2, to_tsvector('simple', $3))
            RETURNING *"#,
        )
        .bind(&title)
        .bind(Json(tags))
        .bind(kw.join("&"))
        .fetch_one(&self.0)
        .await?;

        event!(Level::DEBUG, post_id = new_post.id, title = %new_post.title, "成功创建文章元数据");
        Ok(new_post)
    }

    #[instrument(name = "PostMetaReponsitory::update", level = "debug", skip_all, fields(id = %post.id))]
    async fn update(&self, post: PostMetaUpdate) -> Result<PostMeta, ReponsitoryError> {
        let PostMetaUpdate {
            id,
            title,
            tags,
            kw,
        } = post;

        event!(Level::DEBUG, post_id = id, title = %title, tags_count = tags.len(), keywords_count = kw.len(), "开始更新文章元数据");

        let updated_post = sqlx::query_as::<_, PostMeta>(
            r#"UPDATE
        post SET title = $1, tags = $2,kw = to_tsvector('simple', $3),
        last_modify = CURRENT_TIMESTAMP, count = count+1
        WHERE id = $4 RETURNING *"#,
        )
        .bind(&title)
        .bind(Json(tags))
        .bind(kw.join("&"))
        .bind(id)
        .fetch_one(&self.0)
        .await?;

        event!(Level::DEBUG, post_id = id, title = %updated_post.title, "成功更新文章元数据");
        Ok(updated_post)
    }

    #[instrument(name = "PostMetaReponsitory::delete", level = "debug", skip(self))]
    async fn delete(&self, id: i32) -> Result<(), ReponsitoryError> {
        event!(Level::DEBUG, post_id = id, "开始删除文章元数据");

        let deleted_post =
            sqlx::query_as::<_, PostMeta>("DELETE FROM post WHERE id = $1 RETURNING *")
                .bind(id)
                .fetch_one(&self.0)
                .await?;

        event!(Level::DEBUG, post_id = id, title = %deleted_post.title, "成功删除文章元数据");
        Ok(())
    }
    #[instrument(
        name = "PostMetaReponsitory::list_all_tags",
        level = "debug",
        skip(self)
    )]
    async fn list_all_tags(&self) -> Result<Vec<String>, ReponsitoryError> {
        event!(Level::DEBUG, "开始查询所有标签");

        let tags: Vec<String> = sqlx::query(
            "SELECT DISTINCT jsonb_array_elements_text(tags) as tag FROM post ORDER BY tag",
        )
        .fetch_all(&self.0)
        .await?
        .into_iter()
        .map(|row| row.get::<String, _>("tag"))
        .collect();

        event!(Level::DEBUG, tags_count = tags.len(), "成功查询所有标签");
        Ok(tags)
    }
}
