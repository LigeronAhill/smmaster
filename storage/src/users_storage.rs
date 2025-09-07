use anyhow::{Result, anyhow};
use bson::doc;
use futures::TryStreamExt;
use shared::models::{ListUsersResult, Role, User};
const USERS_COLLECTION: &str = "users";

#[derive(Clone, Debug)]
pub struct UsersStorage {
    collection: mongodb::Collection<User>,
}
impl UsersStorage {
    pub fn new(db: mongodb::Database) -> Self {
        let collection = db.collection(USERS_COLLECTION);
        Self { collection }
    }
    pub async fn create(&self, user: &User) -> Result<Option<User>> {
        self.collection.insert_one(user).await?;
        let inserted = self.get(user.telegram_id).await?;
        Ok(inserted)
    }
    pub async fn get(&self, id: i64) -> Result<Option<User>> {
        let res = self.collection.find_one(doc! {"telegram_id": id}).await?;
        Ok(res)
    }
    pub async fn list_users(
        &self,
        page: u32,
        page_size: u32,
        role: Option<Role>,
        sort_by_created_asc: bool,
    ) -> Result<ListUsersResult> {
        let mut result = Vec::new();
        let filter = if let Some(user_role) = role {
            doc! {
                "role": user_role.to_string(),
            }
        } else {
            doc! {}
        };

        let sort = doc! {
            "created_at": if sort_by_created_asc { 1 } else { -1 }
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
            .sort(sort)
            .limit(page_size as i64)
            .skip(offset as u64)
            .await?;
        while let Some(post) = cursor.try_next().await? {
            result.push(post);
        }
        let lur = ListUsersResult {
            users: result,
            total_count: total_count as u32,
            current_page: page,
            total_pages,
        };
        Ok(lur)
    }
    pub async fn update(&self, user: &User) -> Result<Option<User>> {
        let filter = doc! {
            "_id": user.id,
        };
        let update = doc! {
            "$set": doc! {
              "first_name": &user.first_name,
              "last_name": user.last_name.as_ref(),
              "username": &user.username,
              "language_code": &user.language_code,
              "role": user.role.to_string(),
              "updated_at": bson::DateTime::from(user.updated_at),
              "last_activity": bson::DateTime::from(user.last_activity),
            }
        };
        let updated = self
            .collection
            .find_one_and_update(filter, update)
            .return_document(mongodb::options::ReturnDocument::After)
            .await?;
        Ok(updated)
    }
    pub async fn delete(&self, id: i64) -> Result<()> {
        let query = doc! {
            "telegram_id": id,
        };
        let res = self.collection.delete_one(query).await?;
        if res.deleted_count == 0 {
            return Err(anyhow!("document not found"));
        }
        Ok(())
    }
}
