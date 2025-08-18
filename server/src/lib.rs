use grpc::smm::{self, GetUserRoleResponse, RegisterNewUserResponse};

#[derive(Default)]
pub struct SmmServer {}

#[tonic::async_trait]
impl smm::smm_service_server::SmmService for SmmServer {
    #[doc = " Регистрирует нового пользователя на основе данных из Telegram"]
    async fn register_new_user(
        &self,
        request: tonic::Request<smm::RegisterNewUserRequest>,
    ) -> tonic::Result<tonic::Response<smm::RegisterNewUserResponse>> {
        let _ = request;
        let res = RegisterNewUserResponse { created_user: None };
        Ok(tonic::Response::new(res))
    }

    #[doc = " Возвращает роль пользователя в системе"]
    async fn get_user_role(
        &self,
        request: tonic::Request<smm::GetUserRoleRequest>,
    ) -> tonic::Result<tonic::Response<smm::GetUserRoleResponse>> {
        let _ = request;
        let res = GetUserRoleResponse {
            user_exists: false,
            user_role: 0,
        };
        Ok(tonic::Response::new(res))
    }
}
impl SmmServer {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = std::env::var("GRPC_SERVER_ADDR")?;
        let reflection_service_v1 = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
            .build_v1()?;
        let reflection_service_alpha = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(smm::FILE_DESCRIPTOR_SET)
            .build_v1alpha()?;
        tonic::transport::Server::builder()
            .add_service(reflection_service_v1)
            .add_service(reflection_service_alpha)
            .add_service(smm::smm_service_server::SmmServiceServer::new(self))
            .serve(addr.parse()?)
            .await?;
        Ok(())
    }
}
