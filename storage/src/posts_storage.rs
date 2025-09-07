use anyhow::{Result, anyhow};
use bson::doc;
use futures::TryStreamExt;
use shared::models::{ListPostsResult, Post};
use uuid::Uuid;
const POSTS_COLLECTION: &str = "posts";

#[derive(Clone, Debug)]
pub struct PostsStorage {
    collection: mongodb::Collection<Post>,
}

impl PostsStorage {
    pub fn new(db: mongodb::Database) -> Self {
        let collection = db.collection(POSTS_COLLECTION);
        Self { collection }
    }
    pub async fn create(&self, post: &Post) -> Result<Option<Post>> {
        self.collection.insert_one(post).await?;
        let inserted = self.collection.find_one(doc! {"_id": post.id}).await?;
        Ok(inserted)
    }
    pub async fn get(&self, id: Uuid) -> Result<Option<Post>> {
        let res = self.collection.find_one(doc! {"_id": id}).await?;
        Ok(res)
    }
    pub async fn list_posts(
        &self,
        author_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<ListPostsResult> {
        let mut result = Vec::new();
        let filter = doc! {
            "author_id": author_id,
        };
        let total_count = self.collection.count_documents(filter.clone()).await?;
        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64 - 1.0) / page_size as f64).ceil() as u32 + 1
        };
        let offset = (page - 1) * page_size;
        let mut cursor = self
            .collection
            .find(filter)
            .limit(page_size as i64)
            .skip(offset as u64)
            .await?;
        while let Some(post) = cursor.try_next().await? {
            result.push(post);
        }
        let lpr = ListPostsResult {
            posts: result,
            total_count: total_count as u32,
            current_page: page,
            total_pages,
        };
        Ok(lpr)
    }
    pub async fn update(&self, post: &Post) -> Result<Option<Post>> {
        let query = doc! {
            "_id": post.id,
        };
        let res = self.collection.replace_one(query.clone(), post).await?;
        if res.modified_count == 0 {
            return Err(anyhow!("document not found"));
        }
        let updated = self.collection.find_one(query).await?;
        Ok(updated)
    }
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let query = doc! {
            "_id": id,
        };
        let res = self.collection.delete_one(query).await?;
        if res.deleted_count == 0 {
            return Err(anyhow!("document not found"));
        }
        Ok(())
    }
}
