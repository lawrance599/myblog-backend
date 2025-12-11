use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use tracing::instrument;

use crate::models::post::*;
use crate::repositories::post::{Post, PostReponsitory};
use sqlx::PgPool;
pub struct PostService {
    repository: PostReponsitory,
    save_path: String,
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
        tracing::info!("创建PostServie 实例成功, 保存路径为: {}", save_dir);
        PostService {
            repository: PostReponsitory::new(pool),
            save_path: save_dir.to_string(),
        }
    }

    pub async fn build_file_path(&self, title: &str) -> PathBuf {
        Path::new(&self.save_path).join(title)
    }

    #[instrument(
        skip_all,
        level= "info"
        name="创建新文件"
        fields(
            title
        )
    )]
    async fn save_post(&self, title: &str, content: Vec<u8>) -> Result<(), String> {
        let path = self.build_file_path(title).await;

        let file = match File::create_new(path.as_path()).await {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };
        {
            let mut writer = io::BufWriter::new(file);
            writer.write(&content).await.map_err(|e| e.to_string())?;
            writer.flush().await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    #[instrument(
        name = "添加博文",
        level = "info",
        skip_all,
        fields(
            title = %post.title,
            id
        )
    )]
    pub async fn add_post(&self, post: PostCreate) -> Result<Post, String> {
        let PostCreate {
            title,
            tags,
            content,
        } = post;
        let new = self
            .repository
            .insert_one(title.clone(), tags)
            .await
            .map_err(|e| e.to_string())?;

        tracing::Span::current().record("id", &new.id);

        self.save_post(&title, content).await?;

        Ok(new)
    }
    #[instrument(name = "读取博文元数据", level = "info", skip(self))]
    pub async fn read_post(&self, id: i32) -> Result<Post, String> {
        Ok(self
            .repository
            .find_by_id(id)
            .await
            .map_err(|e| e.to_string())?)
    }
    #[instrument(name = "删除博文", level = "info", skip(self))]
    pub async fn delete_post(&self, id: i32) -> Result<Post, String> {
        Ok(self
            .repository
            .delete_by_id(id)
            .await
            .map_err(|e| e.to_string())?)
    }
    pub async fn list_posts(&self) -> Result<Vec<Post>, String> {
        Ok(self
            .repository
            .find_all(8)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|item| item.into())
            .collect())
    }
}
