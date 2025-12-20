use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::models::{Pagenigation, post::*};
use crate::repositories::post;
use crate::repositories::post::{PostMeta, PostMetaCreate, PostMetaReponsitory};
use crate::service::ServiceError;
use jieba_rs::Jieba;
use sqlx::PgPool;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use tokio::task;
use tracing::{instrument, event, Level};
pub struct PostService {
    post: Box<dyn PostMetaReponsitory>,
    save_path: String,
    jieba: Arc<Jieba>,
}
impl PostService {
    pub fn new(pool: PgPool, save_dir: &str) -> Self {
        let dir = Path::new(save_dir);
        if !dir.exists() {
            if let Err(e) = create_dir_all(dir) {
                panic!(
                    "无法创建文件夹: {}\n{}",
                    dir.to_str().unwrap_or(""),
                    e.to_string()
                )
            }
        }

        if !dir.is_dir() {
            panic!(r"`{}`为无效的路径, 请输入路径", dir.to_str().unwrap_or(""))
        }
        let jieba = Arc::new(Jieba::new());

        tracing::info!("创建PostServie 实例成功, 保存路径为: {}", save_dir);

        PostService {
            post: Box::new(post::SqlxReponsitory::new(pool.clone())),
            save_path: save_dir.to_string(),
            jieba,
        }
    }

    #[instrument(name = "PostService::add_one", level = "info", skip_all, fields(id))]
    pub async fn add_one(&self, post: PostCreate) -> Result<PostMeta, ServiceError> {
        let PostCreate {
            title,
            tags,
            content,
        } = post;

        event!(Level::INFO, title = %title, tags_count = tags.len(), content_size = content.len(), "开始创建新文章");

        //metadata的存储
        // 使用jieba进行分词
        let kw = self.cut(&title).await;
        event!(Level::DEBUG, keywords_count = kw.len(), "完成文章分词");
        
        let post_meta_create = PostMetaCreate { title: title.clone(), tags, kw };
        let new = self.post.add(post_meta_create).await?;

        tracing::Span::current().record("id", &new.id);
        event!(Level::INFO, post_id = new.id, title = %new.title, "成功保存文章元数据");

        // 文件的存储
        self.to_file(&new.title, content).await?;
        event!(Level::INFO, post_id = new.id, title = %new.title, "成功保存文章内容");

        Ok(new)
    }
    #[instrument(name = "PostService::read_one", level = "info", skip(self))]
    pub async fn read_one(&self, id: i32) -> Result<PostMeta, ServiceError> {
        event!(Level::INFO, post_id = id, "开始查询文章元数据");
        let post = self.post.find_by_id(id).await?;
        event!(Level::INFO, post_id = id, title = %post.title, "成功查询文章元数据");
        Ok(post)
    }
    #[instrument(name = "PostService::delete_one", level = "info", skip(self))]
    pub async fn delete_one(&self, id: i32) -> Result<PostMeta, ServiceError> {
        event!(Level::INFO, post_id = id, "开始删除文章");
        
        // 首先获取要删除的 post
        let post = self.post.find_by_id(id).await?;
        event!(Level::INFO, post_id = id, title = %post.title, "找到要删除的文章");

        // 然后删除它
        self.post.delete(id).await?;
        event!(Level::INFO, post_id = id, title = %post.title, "成功删除文章元数据");

        Ok(post)
    }
    #[instrument(name = "PostService::list", level = "info", skip_all, fields(cursor = %page.cursor.unwrap_or_default(), page_size = %page.page_size))]
    pub async fn list(&self, page: Pagenigation) -> Result<Vec<PostMeta>, ServiceError> {
        // TODO: 目前的分页是简单的使用上一次查询的结果的最后一个id作为下一次查询的游标
        // 但是之后考虑将排序方法, 结果id, 用户信息等内容放入cursor后进行加密作为验证手段
        let Pagenigation { cursor, page_size } = page;

        event!(Level::INFO, cursor = ?cursor, page_size = page_size, "开始分页查询文章列表");
        
        let posts = self
            .post
            .list_pagenigation(cursor.unwrap_or(0), page_size)
            .await?;
            
        event!(Level::INFO, post_count = posts.len(), "成功分页查询文章列表");
        Ok(posts)
    }

    async fn cut(&self, text: &str) -> Vec<String> {
        let jieba = Arc::clone(&self.jieba);
        let text = text.to_string();
        
        event!(Level::DEBUG, text_length = text.len(), "开始分词处理");
        
        task::spawn_blocking(move || {
            jieba
                .cut_for_search(&text, true)
                .into_iter()
                .map(|item| item.to_string())
                .collect()
        })
        .await
        .unwrap_or_else(|e| {
            event!(Level::ERROR, error = %e, "分词任务执行失败");
            Vec::new()
        })
    }

    pub async fn build_file_path(&self, title: &str) -> PathBuf {
        Path::new(&self.save_path).join(title)
    }

    #[instrument(
        skip_all,
        level= "info"
        name="创建新文件"
        fields(
            file_name = %file_name,
        )
    )]
    async fn to_file(&self, file_name: &str, content: Vec<u8>) -> Result<(), ServiceError> {
        let path = self.build_file_path(file_name).await;

        event!(Level::DEBUG, file_path = %path.display(), content_size = content.len(), "开始写入文件");

        let file = File::create_new(path.as_path()).await?;
        {
            let mut writer = io::BufWriter::new(file);
            writer.write(&content).await?;
            writer.flush().await?;
        }
        
        event!(Level::DEBUG, file_path = %path.display(), "成功写入文件");
        Ok(())
    }
}
