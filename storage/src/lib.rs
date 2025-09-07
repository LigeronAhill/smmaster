mod posts_storage;
mod users_storage;

use std::sync::Arc;

use anyhow::Result;

const DATABASE: &str = "smmaster";

#[derive(Clone, Debug)]
pub struct Storage {
    users_storage: Arc<users_storage::UsersStorage>,
    posts_storage: Arc<posts_storage::PostsStorage>,
}
impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let db = client.database(DATABASE);
        db.run_command(bson::doc! {"ping": 1}).await?;
        let users_storage = Arc::new(users_storage::UsersStorage::new(db.clone()));
        let posts_storage = Arc::new(posts_storage::PostsStorage::new(db));
        Ok(Self {
            users_storage,
            posts_storage,
        })
    }
    pub fn users(&self) -> Arc<users_storage::UsersStorage> {
        self.users_storage.clone()
    }
    pub fn posts(&self) -> Arc<posts_storage::PostsStorage> {
        self.posts_storage.clone()
    }
}
