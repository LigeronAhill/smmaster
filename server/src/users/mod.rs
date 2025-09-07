use grpc::smm::users;
use tracing::instrument;

#[derive(Debug)]
pub struct AppUsersService {
    db: storage::Storage,
}

impl AppUsersService {
    pub fn new(db: storage::Storage) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl users::users_service_server::UsersService for AppUsersService {
    #[doc = " Создает нового пользователя на основе данных из Telegram"]
    #[instrument(name = "create user", skip(self))]
    async fn create_user(
        &self,
        request: tonic::Request<users::CreateUserRequest>,
    ) -> tonic::Result<tonic::Response<users::CreateUserResponse>> {
        tracing::info!("received request");
        let r = request.into_inner();
        let id = r.telegram_id;
        let created_user = match self
            .db
            .users()
            .get(id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
        {
            Some(existing) => Some(existing.into()),
            None => {
                let new_user: shared::models::User =
                    r.clone().try_into().map_err(|e: anyhow::Error| {
                        tonic::Status::new(tonic::Code::InvalidArgument, e.to_string())
                    })?;
                self.db
                    .users()
                    .create(&new_user)
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?
                    .map(|u| u.into())
            }
        };

        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::CreateUserResponse {
            created_user,
        }))
    }

    #[doc = " Получает информацию о пользователе по его Telegram ID"]
    #[instrument(name = "get user", skip(self))]
    async fn get_user(
        &self,
        request: tonic::Request<users::GetUserRequest>,
    ) -> tonic::Result<tonic::Response<users::GetUserResponse>> {
        tracing::info!("received request");
        let r = request.into_inner();
        let id = r.user_id;
        let user = self
            .db
            .users()
            .get(id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .map(|u| u.into());
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::GetUserResponse { user }))
    }

    #[doc = " Возвращает список пользователей с возможностью пагинации"]
    #[instrument(name = "list users", skip(self))]
    async fn list_users(
        &self,
        request: tonic::Request<users::ListUsersRequest>,
    ) -> tonic::Result<tonic::Response<users::ListUsersResponse>> {
        tracing::info!("received request");
        let r = request.into_inner();
        let page = r.page;
        let page_size = r.page_size;
        if page == 0 || page_size < 10 || page_size > 100 {
            return Err(tonic::Status::invalid_argument("wrong page or page_size"));
        }
        let role = r.role_filter.and_then(|r| r.try_into().ok());
        let sort_by_created_asc = r.sort_by_created_asc();
        let res = self
            .db
            .users()
            .list_users(page, page_size, role, sort_by_created_asc)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into();
        tracing::debug!("sending response");
        Ok(tonic::Response::new(res))
    }

    #[doc = " Обновляет данные пользователя"]
    #[instrument(name = "update user", skip(self))]
    async fn update_user(
        &self,
        request: tonic::Request<users::UpdateUserRequest>,
    ) -> tonic::Result<tonic::Response<users::UpdateUserResponse>> {
        tracing::info!("received request");
        let updated_user = if let Some(update) = request
            .into_inner()
            .updated_user
            .and_then(|u| u.try_into().ok())
        {
            self.db
                .users()
                .update(&update)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
                .map(|u| u.into())
        } else {
            None
        };
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::UpdateUserResponse {
            updated_user,
        }))
    }

    #[doc = " Удаляет пользователя из системы"]
    #[instrument(name = "delete user", skip(self))]
    async fn delete_user(
        &self,
        request: tonic::Request<users::DeleteUserRequest>,
    ) -> tonic::Result<tonic::Response<users::DeleteUserResponse>> {
        tracing::info!("received request");
        let id = request.into_inner().user_id;
        let success = self.db.users().delete(id).await.is_ok();
        tracing::debug!("sending response");
        Ok(tonic::Response::new(users::DeleteUserResponse { success }))
    }
}
