use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::models::post::*;
use crate::repositories::post::{PostMeta, PostMetaCreate, PostMetaReponsitory, SqlxReponsitory};
use crate::service::ServiceError;
use jieba_rs::Jieba;
use sqlx::PgPool;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use tracing::instrument;
pub struct PostService {
    repository: Box<dyn PostMetaReponsitory>,
    save_path: String,
    jieba: Jieba,
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
        let jieba = Jieba::new();

        tracing::info!("创建PostServie 实例成功, 保存路径为: {}", save_dir);

        PostService {
            repository: Box::new(SqlxReponsitory::new(pool)),
            save_path: save_dir.to_string(),
            jieba,
        }
    }

    #[instrument(name = "添加博文", level = "info", skip_all, fields(id))]
    pub async fn add_one(&self, post: PostCreate) -> Result<PostMeta, ServiceError> {
        let PostCreate {
            title,
            tags,
            content,
        } = post;

        //metadata的存储
        // 使用jieba进行分词
        let kw = self.cut(&title).await;
        let post_meta_create = PostMetaCreate { title, tags, kw };
        let new = self.repository.add(post_meta_create).await?;

        tracing::Span::current().record("id", &new.id);

        // 文件的存储
        self.to_file(&new.title, content).await?;

        Ok(new)
    }
    #[instrument(name = "读取博文元数据", level = "info", skip(self))]
    pub async fn read_one(&self, id: i32) -> Result<PostMeta, ServiceError> {
        Ok(self.repository.find_by_id(id).await?)
    }
    #[instrument(name = "删除博文", level = "info", skip(self))]
    pub async fn delete_one(&self, id: i32) -> Result<PostMeta, ServiceError> {
        // 首先获取要删除的 post
        let post = self.repository.find_by_id(id).await?;

        // 然后删除它
        self.repository.delete(id).await?;

        Ok(post)
    }
    pub async fn list_all(&self) -> Result<Vec<PostMeta>, ServiceError> {
        Ok(self.repository.list_all().await?)
    }

    async fn cut(&self, text: &str) -> Vec<String> {
        self.jieba
            .cut_for_search(text, true)
            .into_iter()
            .map(|item| item.to_string())
            .collect()
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

        let file = File::create_new(path.as_path()).await?;
        {
            let mut writer = io::BufWriter::new(file);
            writer.write(&content).await?;
            writer.flush().await?;
        }
        Ok(())
    }
}
