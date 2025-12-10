use tokio::io::{self, AsyncWriteExt};
use tokio::fs::File;

use crate::models::post::*;
use sqlx::PgPool;
use crate::repositories::post::{Post, PostReponsitory};
pub struct PostService{
    repository: PostReponsitory
}
impl PostService{
    pub fn new(pool: PgPool) -> Self{
        PostService{
            repository: PostReponsitory::new(pool)
        }
    }
    
    async fn save_post(&self, title: &str, content: Vec<u8>) -> Result<(), String>{ 
        let file = match File::create_new(title).await {
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

    pub async fn add_post(&self, post: PostCreate) -> Result<Post, String>{
        let PostCreate{title, tags, content} = post;
        self.save_post(&title, content).await?;
        Ok(self.repository.insert_post(title, tags)
                .await
                .map_err(|e|e.to_string())?)
    }

    pub async fn get_post(&self, id: i32) -> Result<Post, String>{
        Ok(self.repository.get_post_by_id(id)
                .await
                .map_err(|e|e.to_string())?)
    }
    pub async fn delete_post(&self, id: i32) -> Result<Post, String>{
        Ok(self.repository.delete_post_by_id(id)
                .await
                .map_err(|e|e.to_string())?)
    }
}
