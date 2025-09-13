mod auth;

use anyhow::{Result, anyhow};
use grpc::smm::{
    posts::posts_service_client::PostsServiceClient,
    users::users_service_client::UsersServiceClient,
};
use shared::models::{Post, Status, User};
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(Clone)]
pub struct Client {
    pub users_client: UsersServiceClient<InterceptedService<Channel, auth::Auth>>,
    pub posts_client: PostsServiceClient<InterceptedService<Channel, auth::Auth>>,
}
impl Client {
    #[instrument(name = "new rpc client", skip(token))]
    pub async fn new(port: u16, token: String) -> Result<Self> {
        let addr = format!("http://[::1]:{port}");
        let channel = Channel::builder(addr.parse()?).connect().await?;
        let bearer_token = format!("Bearer {token}");
        let auth = auth::Auth::new(bearer_token)?;
        let users_client =
            grpc::smm::users::users_service_client::UsersServiceClient::with_interceptor(
                channel.clone(),
                auth.clone(),
            );
        let posts_client =
            grpc::smm::posts::posts_service_client::PostsServiceClient::with_interceptor(
                channel, auth,
            );
        info!("rpc client initialized");
        Ok(Self {
            users_client,
            posts_client,
        })
    }

    #[instrument(name = "create user", skip(self))]
    pub async fn create_user(
        &mut self,
        telegram_id: i64,
        first_name: String,
        last_name: Option<String>,
        username: Option<String>,
        language_code: Option<String>,
    ) -> Result<Option<User>> {
        let request = tonic::Request::new(grpc::smm::users::CreateUserRequest {
            telegram_id,
            first_name,
            last_name,
            username,
            language_code,
        });
        let response = self
            .users_client
            .create_user(request)
            .await?
            .into_inner()
            .created_user
            .and_then(|u| u.try_into().ok());
        if let Some(created) = response.as_ref() {
            info!("Created new user:\n{created:#?}");
        }
        Ok(response)
    }

    #[instrument(name = "get user", skip(self))]
    pub async fn get_user(&mut self, user_id: i64) -> Result<Option<User>> {
        let request = tonic::Request::new(grpc::smm::users::GetUserRequest { user_id });
        let response = self
            .users_client
            .get_user(request)
            .await?
            .into_inner()
            .user
            .and_then(|u| u.try_into().ok());
        if let Some(founded) = response.as_ref() {
            info!("Result:\n{founded:#?}");
        }
        Ok(response)
    }
    #[instrument(name = "check if bot has admin", skip(self))]
    pub async fn has_admin(&mut self) -> Result<bool> {
        let request = tonic::Request::new(grpc::smm::users::ListUsersRequest {
            page: 1,
            page_size: 10,
            role_filter: Some(shared::models::Role::Admin.into()),
            sort_by_created_asc: None,
        });
        let response = self
            .users_client
            .list_users(request)
            .await?
            .into_inner()
            .users;
        info!("Total admins: {l}", l = response.len());
        Ok(!response.is_empty())
    }

    #[instrument(name = "list users", skip(self))]
    pub async fn list_users(&mut self, page: u32) -> Result<(Vec<User>, bool)> {
        let request = tonic::Request::new(grpc::smm::users::ListUsersRequest {
            page,
            page_size: 10,
            role_filter: None,
            sort_by_created_asc: None,
        });
        let response = self.users_client.list_users(request).await?.into_inner();
        let total_pages = response.total_pages;
        let has_next = response.total_pages > page;
        info!("Current page: {page}, total pages: {total_pages} => has next: {has_next}");
        let list = response
            .users
            .into_iter()
            .flat_map(|u| u.try_into())
            .collect();
        Ok((list, has_next))
    }

    #[instrument(name = "update user", skip(self))]
    pub async fn update_user(&mut self, user: User) -> Result<Option<User>> {
        let request = tonic::Request::new(grpc::smm::users::UpdateUserRequest {
            updated_user: Some(user.into()),
        });
        let response = self
            .users_client
            .update_user(request)
            .await?
            .into_inner()
            .updated_user
            .and_then(|u| u.try_into().ok());
        Ok(response)
    }

    #[instrument(name = "delete user", skip(self))]
    pub async fn delete_user(&mut self, id: i64) -> Result<bool> {
        let request = tonic::Request::new(grpc::smm::users::DeleteUserRequest { user_id: id });
        let response = self
            .users_client
            .delete_user(request)
            .await?
            .into_inner()
            .success;
        info!("Delete user result: {response}");
        Ok(response)
    }

    #[instrument(name = "get draft posts of user", skip(self))]
    pub async fn drafts(&mut self, author_tg_id: i64, page: u32) -> Result<(Vec<Post>, bool)> {
        let request = tonic::Request::new(grpc::smm::posts::ListPostsRequest {
            author_tg_id,
            page,
            page_size: 10,
            status_filter: Some(Status::Draft.into()),
        });
        let response = self.posts_client.list_posts(request).await?.into_inner();
        let total_pages = response.total_pages;
        let has_next = total_pages > page;
        info!("Current page: {page}, total pages: {total_pages} => has next: {has_next}");
        let posts = response
            .posts
            .into_iter()
            .flat_map(|p| p.try_into())
            .collect();
        Ok((posts, has_next))
    }

    #[instrument(name = "get pending posts of user", skip(self))]
    pub async fn pending(&mut self, author_tg_id: i64, page: u32) -> Result<(Vec<Post>, bool)> {
        let request = tonic::Request::new(grpc::smm::posts::ListPostsRequest {
            author_tg_id,
            page,
            page_size: 10,
            status_filter: Some(Status::Pending.into()),
        });
        let response = self.posts_client.list_posts(request).await?.into_inner();
        let total_pages = response.total_pages;
        let has_next = total_pages > page;
        info!("Current page: {page}, total pages: {total_pages} => has next: {has_next}");
        let posts = response
            .posts
            .into_iter()
            .flat_map(|p| p.try_into())
            .collect();
        Ok((posts, has_next))
    }

    #[instrument(name = "get published posts of user", skip(self))]
    pub async fn published(&mut self, author_tg_id: i64, page: u32) -> Result<(Vec<Post>, bool)> {
        let request = tonic::Request::new(grpc::smm::posts::ListPostsRequest {
            author_tg_id,
            page,
            page_size: 10,
            status_filter: Some(Status::Published.into()),
        });
        let response = self.posts_client.list_posts(request).await?.into_inner();
        let total_pages = response.total_pages;
        let has_next = total_pages > page;
        info!("Current page: {page}, total pages: {total_pages} => has next: {has_next}");
        let posts = response
            .posts
            .into_iter()
            .flat_map(|p| p.try_into())
            .collect();
        Ok((posts, has_next))
    }

    #[instrument(name = "create new post", skip(self))]
    pub async fn create_post(
        &mut self,
        author_tg_id: i64,
        title: String,
        content: String,
        tg_photo_file_id: Option<String>,
        vk_photo_file_id: Option<String>,
        tg_video_file_id: Option<String>,
        vk_video_file_id: Option<String>,
    ) -> Result<Post> {
        let request = tonic::Request::new(grpc::smm::posts::CreatePostRequest {
            author_tg_id,
            title,
            content,
            tg_photo_file_id,
            vk_photo_file_id,
            tg_video_file_id,
            vk_video_file_id,
            publish_datetime: None,
        });
        let response = self
            .posts_client
            .create_post(request)
            .await?
            .into_inner()
            .created_post
            .and_then(|p| p.try_into().ok())
            .ok_or(anyhow!("Error creating post"))?;
        info!("Created post:\n{response:#?}");
        Ok(response)
    }

    #[instrument(name = "delete post", skip(self))]
    pub async fn delete_post(&mut self, post_id: Uuid) -> Result<()> {
        let request = tonic::Request::new(grpc::smm::posts::DeletePostRequest {
            post_id: post_id.into(),
        });
        let response = self
            .posts_client
            .delete_post(request)
            .await?
            .into_inner()
            .success;
        info!("Delete post result: {response}");
        if response {
            Ok(())
        } else {
            Err(anyhow!("Error deleting post"))
        }
    }

    #[instrument(name = "get post", skip(self))]
    pub async fn get_post(&mut self, post_id: Uuid) -> Result<Option<Post>> {
        let request = tonic::Request::new(grpc::smm::posts::GetPostRequest {
            post_id: post_id.into(),
        });
        let response = self
            .posts_client
            .get_post(request)
            .await?
            .into_inner()
            .post
            .and_then(|p| p.try_into().ok());
        if let Some(founded) = response.as_ref() {
            info!("Found post:\n{founded:#?}");
        }
        Ok(response)
    }

    #[instrument(name = "set post published", skip(self))]
    pub async fn publish_now(&mut self, post_id: Uuid) -> Result<Option<Post>> {
        let Some(mut existing) = self.get_post(post_id).await? else {
            return Err(anyhow!("post not found"));
        };
        existing.publish_datetime = Some(chrono::Utc::now());
        existing.status = Status::Published;
        let request = tonic::Request::new(grpc::smm::posts::UpdatePostRequest {
            updated_post: Some(existing.into()),
        });
        let response = self
            .posts_client
            .update_post(request)
            .await?
            .into_inner()
            .updated_post
            .and_then(|p| p.try_into().ok());
        if let Some(updated) = response.as_ref() {
            info!("Updated post:\n{updated:#?}");
        }
        Ok(response)
    }

    #[instrument(name = "set post publish date", skip(self))]
    pub async fn set_publish_date(
        &mut self,
        post_id: Uuid,
        publish_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<Post>> {
        let Some(mut existing) = self.get_post(post_id).await? else {
            return Err(anyhow!("post not found"));
        };
        existing.publish_datetime = Some(publish_date);
        existing.status = Status::Pending;
        let request = tonic::Request::new(grpc::smm::posts::UpdatePostRequest {
            updated_post: Some(existing.into()),
        });
        let response = self
            .posts_client
            .update_post(request)
            .await?
            .into_inner()
            .updated_post
            .and_then(|p| p.try_into().ok());
        if let Some(updated) = response.as_ref() {
            info!("Updated post:\n{updated:#?}");
        }
        Ok(response)
    }
}
