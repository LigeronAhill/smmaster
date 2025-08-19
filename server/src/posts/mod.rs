use grpc::smm::posts::{
    self, CreatePostRequest, CreatePostResponse, DeletePostRequest, DeletePostResponse,
    GetPostRequest, GetPostResponse, ListPostsRequest, ListPostsResponse, UpdatePostRequest,
    UpdatePostResponse,
};
use tonic::{Request, Response, Result};
use tracing::instrument;

#[derive(Default, Debug)]
pub struct AppPostService {}
impl AppPostService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tonic::async_trait]
impl posts::posts_service_server::PostsService for AppPostService {
    #[doc = " Создает новый пост"]
    #[instrument(name = "create post", skip_self)]
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(Response::new(CreatePostResponse { created_post: None }))
    }

    #[doc = " Возвращает пост по идентификатору"]
    #[instrument(name = "get post", skip_self)]
    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<tonic::Response<GetPostResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(Response::new(GetPostResponse { post: None }))
    }

    #[doc = " Возвращает список постов с пагинацией"]
    #[instrument(name = "list posts", skip_self)]
    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(Response::new(ListPostsResponse {
            posts: Vec::new(),
            total_count: 0,
            current_page: 1,
            total_pages: 1,
        }))
    }

    #[doc = " Обновляет существующий пост"]
    #[instrument(name = "update post", skip_self)]
    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<UpdatePostResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(Response::new(UpdatePostResponse { updated_post: None }))
    }

    #[doc = " Удаляет пост"]
    #[instrument(name = "delete post", skip_self)]
    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(Response::new(DeletePostResponse { success: false }))
    }
}
