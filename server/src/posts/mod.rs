use grpc::smm::posts::{
    self, CreatePostRequest, CreatePostResponse, DeletePostRequest, DeletePostResponse,
    GetPostRequest, GetPostResponse, ListPostsRequest, ListPostsResponse, UpdatePostRequest,
    UpdatePostResponse,
};
use tonic::{Request, Response, Result};
use tracing::instrument;

#[derive(Debug)]
pub struct AppPostService {
    db: storage::Storage,
}
impl AppPostService {
    pub fn new(db: storage::Storage) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl posts::posts_service_server::PostsService for AppPostService {
    #[doc = " Создает новый пост"]
    #[instrument(name = "create post", skip(self))]
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>> {
        tracing::info!("received request");
        let post_to_create = request.into_inner();
        let author_id = self
            .db
            .users()
            .get(post_to_create.author_tg_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or(tonic::Status::not_found("author not found"))?
            .id;
        let post = post_to_create
            .convert(author_id.to_string())
            .map_err(|e: anyhow::Error| {
                tonic::Status::new(tonic::Code::InvalidArgument, e.to_string())
            })?;
        let created_post = self
            .db
            .posts()
            .create(&post)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map(|p| p.into());
        tracing::debug!("sending response");
        Ok(Response::new(CreatePostResponse { created_post }))
    }

    #[doc = " Возвращает пост по идентификатору"]
    #[instrument(name = "get post", skip(self))]
    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<tonic::Response<GetPostResponse>> {
        tracing::info!("received request");
        let id = request
            .into_inner()
            .post_id
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("wrong post id"))?;
        let post = self
            .db
            .posts()
            .get(id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map(|p| p.into());
        tracing::debug!("sending response");
        Ok(Response::new(GetPostResponse { post }))
    }

    #[doc = " Возвращает список постов с пагинацией"]
    #[instrument(name = "list posts", skip(self))]
    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>> {
        tracing::info!("received request");
        let l = request.into_inner();
        let author_tg_id = l.author_tg_id;
        if author_tg_id < 0 {
            return Err(tonic::Status::invalid_argument("wrong tg user id"));
        }
        let author_id = self
            .db
            .users()
            .get(author_tg_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map(|u| u.id)
            .ok_or(tonic::Status::not_found("author not found"))?;
        let page = l.page;
        let page_size = l.page_size;
        let filter = l.status_filter.and_then(|s| s.try_into().ok());
        if page == 0 || page_size < 10 || page_size > 100 {
            return Err(tonic::Status::invalid_argument("wrong page or page_size"));
        }
        let resp = self
            .db
            .posts()
            .list_posts(author_id, page, page_size, filter)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into();
        tracing::debug!("sending response");
        Ok(Response::new(resp))
    }

    #[doc = " Обновляет существующий пост"]
    #[instrument(name = "update post", skip(self))]
    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<UpdatePostResponse>> {
        tracing::info!("received request");
        let post = request
            .into_inner()
            .updated_post
            .and_then(|p| p.try_into().ok())
            .ok_or(tonic::Status::invalid_argument("post required"))?;
        let updated_post = self
            .db
            .posts()
            .update(&post)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map(|p| p.into());
        tracing::debug!("sending response");
        Ok(Response::new(UpdatePostResponse { updated_post }))
    }

    #[doc = " Удаляет пост"]
    #[instrument(name = "delete post", skip(self))]
    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>> {
        tracing::info!("received request");
        let id = request
            .into_inner()
            .post_id
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("wrong post id"))?;
        let success = self.db.posts().delete(id).await.is_ok();
        tracing::debug!("sending response");
        Ok(Response::new(DeletePostResponse { success }))
    }
}
