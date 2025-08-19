use grpc::smm::users;
use tracing::instrument;

#[derive(Default, Debug)]
pub struct AppUsersService {}

impl AppUsersService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tonic::async_trait]
impl users::users_service_server::UsersService for AppUsersService {
    #[doc = " Создает нового пользователя на основе данных из Telegram"]
    #[instrument(name = "create user", skip_self)]
    async fn create_user(
        &self,
        request: tonic::Request<users::CreateUserRequest>,
    ) -> tonic::Result<tonic::Response<users::CreateUserResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::CreateUserResponse {
            created_user: None,
        }))
    }

    #[doc = " Получает информацию о пользователе по его Telegram ID"]
    #[instrument(name = "get user", skip_self)]
    async fn get_user(
        &self,
        request: tonic::Request<users::GetUserRequest>,
    ) -> tonic::Result<tonic::Response<users::GetUserResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::GetUserResponse { user: None }))
    }

    #[doc = " Возвращает список пользователей с возможностью пагинации"]
    #[instrument(name = "list users", skip_self)]
    async fn list_users(
        &self,
        request: tonic::Request<users::ListUsersRequest>,
    ) -> tonic::Result<tonic::Response<users::ListUsersResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::ListUsersResponse {
            users: Vec::new(),
            total_count: 0,
            current_page: 1,
            total_pages: 1,
        }))
    }

    #[doc = " Обновляет данные пользователя"]
    #[instrument(name = "update user", skip_self)]
    async fn update_user(
        &self,
        request: tonic::Request<users::UpdateUserRequest>,
    ) -> tonic::Result<tonic::Response<users::UpdateUserResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::UpdateUserResponse {
            updated_user: None,
        }))
    }

    #[doc = " Удаляет пользователя из системы"]
    #[instrument(name = "delete user", skip_self)]
    async fn delete_user(
        &self,
        request: tonic::Request<users::DeleteUserRequest>,
    ) -> tonic::Result<tonic::Response<users::DeleteUserResponse>> {
        tracing::info!("received request");
        let _ = request;
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::DeleteUserResponse {
            success: false,
        }))
    }
}
